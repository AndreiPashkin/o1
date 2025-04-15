//! Compile-time implementation of Xorshift PRNG for 16, 32, and 64-bit integers, based on
//! [Marsaglia (2003)] and [Metcalf (n.d.)].
//!
//! [Marsaglia (2003)]: https://www.jstatsoft.org/article/view/v008i14
//! [Metcalf (n.d.)]: http://www.retroprogramming.com/2017/07/xorshift-pseudorandom-numbers-in-z80.html
pub struct XorShift<T: Default + Copy> {
    state: T,
}

impl XorShift<u16> {
    pub const fn new(seed: u16) -> Self {
        debug_assert!(seed != 0, r#""seed" must be non-zero"#);

        XorShift { state: seed }
    }

    pub const fn next(&mut self) -> u16 {
        let mut x = self.state;
        x ^= x << 7;
        x ^= x >> 9;
        x ^= x << 8;
        self.state = x;
        x
    }
}

impl XorShift<u32> {
    pub const fn new(seed: u32) -> Self {
        debug_assert!(seed != 0, r#""seed" must be non-zero"#);

        XorShift { state: seed }
    }

    pub const fn next(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }
}

impl XorShift<u64> {
    pub const fn new(seed: u64) -> Self {
        debug_assert!(seed != 0, r#""seed" must be non-zero"#);

        XorShift { state: seed }
    }

    pub const fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

macro_rules! generate_random {
    ($ty:ty, $seed:expr) => {{
        use crate::random::xorshift::XorShift;

        let mut rng = XorShift::<$ty>::new($seed);
        rng.next()
    }};
}
pub(crate) use generate_random;

macro_rules! generate_random_array {
    ($ty:ty, $len:expr, $seed:expr) => {{
        use crate::random::xorshift::XorShift;

        let mut rng = XorShift::<$ty>::new($seed);
        let mut arr: [$ty; $len] = [0; $len];
        let mut i = 0;
        while i < $len {
            arr[i] = rng.next();
            i += 1;
        }
        arr
    }};
}
pub(crate) use generate_random_array;
