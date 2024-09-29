#[test]
fn try_all() {
    let t = trybuild::TestCases::new();
    t.pass("tests/trybuild/pass/*.rs");

    #[cfg(feature = "html_escape")]
    t.compile_fail("tests/trybuild/fail/should_fail_when_html_escape_is_enabled_*.rs");

    #[cfg(not(feature = "html_escape"))]
    t.compile_fail("tests/trybuild/fail/should_fail_when_html_esape_is_disabled_*.rs");

    #[cfg(feature = "minify_html")]
    t.compile_fail("tests/trybuild/fail/should_fail_when_minify_html_is_enabled_*.rs");

    #[cfg(not(feature = "minify_html"))]
    t.compile_fail("tests/trybuild/fail/should_fail_when_minify_html_is_disabled_*.rs");
}
