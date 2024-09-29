#[test]
fn try_all() {
    let t = trybuild::TestCases::new();
    t.pass("tests/trybuild/pass/*.rs");
    #[cfg(not(feature = "html_escape"))]
    t.compile_fail("tests/trybuild/fail/should_fail_when_html_esape_is_disabled_*.rs");
}
