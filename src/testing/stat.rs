//! Implements various empirical tests for testing hash functions.
use crate::testing::{Generate, Jitter};
use ndarray::prelude::*;
use ndarray::{ScalarOperand, Zip};
use num_traits::{Float, FromPrimitive, Num, NumAssignOps, ToPrimitive};
use rand::prelude::*;
use statrs::distribution::{ChiSquared, ContinuousCDF};
use std::fmt::Debug;

/// Creates a contingency matrix from two equally sized 1-D arrays.
pub fn make_contingency_matrix<T, C>(
    x: &Array1<T>,
    y: &Array1<T>,
    num_categories: usize,
) -> Array2<C>
where
    T: Num + ToPrimitive,
    C: Num + ToPrimitive + NumAssignOps + From<f32> + Copy,
{
    debug_assert_eq!(x.len(), y.len(), r#""x" and "y" must have equal length"#);
    debug_assert!(num_categories > 0, "Must have at least 1 category");

    let mut contingency = Array2::<C>::zeros((num_categories, num_categories));
    Zip::from(x).and(y).for_each(|xv, yv| {
        let xix: usize = xv.to_usize().unwrap();
        let yix: usize = yv.to_usize().unwrap();

        debug_assert!(
            xix < num_categories,
            r#""x" value {} exceeds "num_categories" {}"#,
            xix,
            num_categories
        );
        debug_assert!(
            yix < num_categories,
            r#""y" value {} exceeds "num_categories" {}"#,
            yix,
            num_categories
        );

        contingency[[xix, yix]] += 1.0_f32.into();
    });
    if contingency.iter().any(|x| *x == 0.0_f32.into()) {
        contingency.mapv_inplace(|x| x + 0.1_f32.into());
    }
    contingency
}

/// An aggregation of p-values from multiple runs of a statistical test.
#[derive(Debug)]
pub struct PValueAggregation {
    pub outcome: bool,
    pub alpha: f64,
    pub expected_passes: f64,
    pub num_passes: usize,
    pub uniformity: Chi2Statistic<f64>,
}

/// Aggregates test results from multiple runs of a statistical test.
///
/// # Notes
///
/// Based on [Bassham et al. (2010)] which describes approaches for statistically
/// testing PRNGs and in particular - approaches to aggregation of the test results
/// (see section 4.2).
///
/// [Bassham et al. (2010)]: https://doi.org/10.6028/NIST.SP.800-22r1a
pub fn aggregate_p_values<'a, V, A>(p_values: A, alpha: f64) -> PValueAggregation
where
    V: 'a + Float + NumAssignOps + From<f64> + ScalarOperand,
    A: AsArray<'a, V>,
{
    let p_values = p_values.into();
    let num_trials = p_values.len() as f64;
    let pass_rate = 1.0 - alpha;
    let confidence = 3.0 * ((pass_rate * (1.0 - pass_rate)) / num_trials).sqrt();
    let expected_passes = num_trials * (pass_rate - confidence);
    let num_passes = p_values
        .iter()
        .fold(0, |acc, &p| if p >= alpha.into() { acc + 1 } else { acc });

    let passes_outcome = num_passes as f64 >= expected_passes;

    let p_values_f64 = p_values.mapv(|x| x.to_f64().unwrap());

    let uniformity = chi2_uniformity(&p_values_f64);

    let uniformity_outcome = uniformity.p_value > alpha;

    let outcome = passes_outcome && uniformity_outcome;

    PValueAggregation {
        outcome,
        alpha,
        expected_passes,
        num_passes,
        uniformity,
    }
}

/// A result of a Chi-square test.
#[derive(Debug)]
pub struct Chi2Statistic<V> {
    pub chi2: V,
    pub dof: usize,
    pub p_value: V,
}

/// Calculates the chi-square statistic.
pub fn chi2<V>(observed: &[V], expected: &[V], dof: Option<usize>) -> Chi2Statistic<V>
where
    V: Float + NumAssignOps + From<f64>,
{
    debug_assert_eq!(observed.len(), expected.len(), "Dimensions must match");
    let chi2: V = Zip::from(observed)
        .and(expected)
        .fold(0.0.into(), |acc, &obs, &exp| {
            let diff = obs - exp;
            acc + diff.powf(2.0.into()) / exp
        });

    let dof = if let Some(dof) = dof {
        dof
    } else {
        observed.len() - 1
    };
    let dist = ChiSquared::new(dof as f64).unwrap();
    let p_value = (1.0 - dist.cdf(chi2.to_f64().unwrap())).into();

    Chi2Statistic { chi2, dof, p_value }
}

/// Performs a Chi-square independence test.
pub fn chi2_independence<V>(contingency: &Array2<V>) -> Chi2Statistic<V>
where
    V: Float + NumAssignOps + From<f64> + ScalarOperand,
{
    let row_sums = contingency.sum_axis(Axis(1));
    let col_sums = contingency.sum_axis(Axis(0));
    let total_sum = row_sums.sum();

    let expected = (&row_sums.insert_axis(Axis(1)) * &col_sums.insert_axis(Axis(0))) / total_sum;
    let dof = (contingency.nrows() - 1) * (contingency.ncols() - 1);
    chi2(
        contingency.as_slice().unwrap(),
        expected.as_slice().unwrap(),
        Some(dof),
    )
}

/// Performs a Chi-square uniformity test.
pub fn chi2_uniformity<'a, V, A>(observed: A) -> Chi2Statistic<V>
where
    V: Float + NumAssignOps + From<f64> + ScalarOperand,
    A: AsArray<'a, V>,
{
    let observed: ArrayView1<V> = observed.into();
    let total_sum = observed.sum();
    let num_cells = observed.len();
    let expected_value = total_sum / (num_cells as f64).into();

    let expected = Array1::<V>::from_elem(observed.dim(), expected_value);

    chi2(
        observed.as_slice().unwrap(),
        expected.as_slice().unwrap(),
        None,
    )
}

/// Mutual information statistic.
#[derive(Debug)]
pub struct MIStatistic {
    pub mi: f64,
    pub bias_corrected_mi: f64,
    pub g: f64,
    pub df: usize,
    pub p_value: f64,
    pub chi2_crit: f64,
}

/// Calculates mutual information based on a contingency table.
pub fn raw_mutual_information<V>(contingency: &Array2<V>) -> f64
where
    V: Num + ToPrimitive + Copy,
{
    let n = contingency.sum().to_f64().unwrap();
    let p_xy = contingency.mapv(|count| count.to_f64().unwrap() / n);

    let p_x = p_xy.sum_axis(Axis(1));
    let p_y = p_xy.sum_axis(Axis(0));

    let ln_px = p_x.mapv(|v| v.ln());
    let ln_py = p_y.mapv(|v| v.ln());

    let mi = p_xy.indexed_iter().fold(0.0, |acc, ((x, y), &p_xyv)| {
        if p_xyv > 0.0 {
            acc + p_xyv * (p_xyv.ln() - (ln_px[x] + ln_py[y]))
        } else {
            acc
        }
    });

    mi
}

/// Calculates mutual information, bias-corrected mutual information, and auxiliary statistics.
pub fn mutual_information<V>(contingency: &Array2<V>) -> MIStatistic
where
    V: Float + FromPrimitive,
{
    let raw_mi = raw_mutual_information(contingency);

    let g = 2.0 * contingency.sum().to_f64().unwrap() * raw_mi;

    let df = (contingency.nrows() - 1) * (contingency.ncols() - 1);
    let dist = ChiSquared::new(df as f64).unwrap();
    let p_value = 1.0 - dist.cdf(g);
    let chi2_crit = dist.inverse_cdf(1.0 - 0.05);

    let row_sums = contingency.sum_axis(Axis(1));
    let col_sums = contingency.sum_axis(Axis(0));
    let total_sum = row_sums.sum();

    let kx = row_sums
        .iter()
        .filter(|&&sum| sum > V::from_f64(0.0).unwrap())
        .count();
    let ky = col_sums
        .iter()
        .filter(|&&sum| sum > V::from_f64(0.0).unwrap())
        .count();

    // Application of Miller-Madow bias correction
    let correction: f64 = ((kx as f64 - 1.0) * (ky as f64 - 1.0))
        / (V::from_f64(2.0).unwrap() * total_sum.max(V::from_f64(1.0).unwrap()))
            .to_f64()
            .unwrap();
    let bias_corrected_mi: f64 = (raw_mi - correction).max(0.0);

    MIStatistic {
        mi: raw_mi,
        bias_corrected_mi,
        g,
        df,
        p_value,
        chi2_crit,
    }
}

/// Tests a hash function family for strong universality.
pub fn strong_universality<R, K>(
    rng: &mut R,
    family: &dyn Fn(&mut R, usize) -> (Box<dyn Fn(&K) -> usize>, usize),
    raw_num_buckets: usize,
    num_samples_per_bucket: u32,
    num_trials: u32,
    alpha: f64,
) where
    R: Rng,
    K: PartialEq + Default + Clone + Generate<R> + Jitter<R> + Debug,
{
    let (_, num_buckets) = family(rng, raw_num_buckets);
    let num_possible_pairs = num_buckets.pow(2);

    let mut independence_statistics = Vec::new();
    let mut uniformity_statistics = Vec::new();
    let mut max_mi = 0.0;

    let mut x = K::generate(rng, &<K as Generate<R>>::GenerateParams::default());
    let mut y: K;

    for _ in 0..num_trials {
        let num_trials = num_samples_per_bucket as usize * num_possible_pairs;
        (x, y) = loop {
            let new_x = x.clone().jitter(rng).unwrap();
            let new_y = x.clone().jitter(rng).unwrap();
            if new_x != new_y {
                break (new_x, new_y);
            }
        };
        let mut hxs = Array1::zeros(num_trials);
        let mut hys = Array1::zeros(num_trials);

        for i in 0..num_trials {
            let (hash_function, _) = family(rng, num_buckets);
            let hx = hash_function(&x);
            let hy = hash_function(&y);
            hxs[i] = hx;
            hys[i] = hy;
        }
        let contingency: Array2<f64> = make_contingency_matrix(&hxs, &hys, num_buckets);
        let independence_statistic = chi2_independence(&contingency);
        independence_statistics.push(independence_statistic);
        let uniformity_statistic = chi2_uniformity(
            contingency
                .view()
                .into_shape_with_order((contingency.len(),))
                .unwrap(),
        );
        uniformity_statistics.push(uniformity_statistic);

        let mi_statistic = mutual_information(&contingency);
        max_mi = max_mi.max(mi_statistic.bias_corrected_mi);
    }

    let independence_p_values = Array1::from_shape_vec(
        independence_statistics.len(),
        independence_statistics.iter().map(|s| s.p_value).collect(),
    )
    .unwrap();
    let uniformity_p_values = Array1::from_shape_vec(
        uniformity_statistics.len(),
        uniformity_statistics.iter().map(|s| s.p_value).collect(),
    )
    .unwrap();

    let independence_result = aggregate_p_values(&independence_p_values, alpha);
    let uniformity_result = aggregate_p_values(&uniformity_p_values, alpha);

    assert!(
        independence_result.outcome,
        "Pairwise independence test has failed:\n{:?}",
        independence_result,
    );
    assert!(
        uniformity_result.outcome,
        "Pairwise uniformity test has failed:\n{:?}",
        uniformity_result,
    );
    // Currently this is more of a sanity check, 0.09 threshold has been chosen based on practice
    // as a guard against hash functions that have serious flaws.
    // TODO: Stricter threshold should be applied.
    assert!(max_mi < 0.09, "Max MI is too high: {}", max_mi);
}
