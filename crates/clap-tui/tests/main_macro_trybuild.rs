#[test]
fn main_macro_covers_valid_and_invalid_usage() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/trybuild/main_macro/pass/*.rs");
    tests.compile_fail("tests/trybuild/main_macro/fail/*.rs");
}
