#[test]
fn derive() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/derive/primitive_types/struct/runtime.rs");
    tests.pass("tests/derive/primitive_types/struct/const.rs");
    tests.pass("tests/derive/primitive_types/enum/runtime.rs");
    tests.pass("tests/derive/primitive_types/enum/const.rs");
    tests.pass("tests/derive/user_defined_types/struct/runtime.rs");
    tests.pass("tests/derive/user_defined_types/struct/const.rs");
    tests.pass("tests/derive/user_defined_types/enum/runtime.rs");
    tests.pass("tests/derive/user_defined_types/enum/const.rs");
    tests.pass("tests/derive/maps/struct/runtime.rs");
    tests.pass("tests/derive/maps/struct/const.rs");
    tests.pass("tests/derive/maps/enum/runtime.rs");
    tests.pass("tests/derive/maps/enum/const.rs");
    tests.pass("tests/derive/skip/struct/runtime.rs");
    tests.pass("tests/derive/skip/struct/const.rs");
    tests.pass("tests/derive/skip/enum/runtime.rs");
    tests.pass("tests/derive/skip/enum/const.rs");
    tests.pass("tests/derive/field_hasher/struct/runtime.rs");
    tests.pass("tests/derive/field_hasher/struct/const.rs");
    tests.pass("tests/derive/field_hasher/enum/runtime.rs");
    tests.pass("tests/derive/field_hasher/enum/const.rs");
}
