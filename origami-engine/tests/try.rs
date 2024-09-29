#[test]
fn try_all() {
    let t = trybuild::TestCases::new();
    t.pass("tests/trybuild/pass/*.rs");
    t.compile_fail("tests/trybuild/fail/*.rs");

    #[cfg(feature = "html_escape")]
    t.pass("tests/trybuild/feauture/pass/*.rs");
    t.compile_fail("tests/trybuild/feature/fail/*.rs");
}
