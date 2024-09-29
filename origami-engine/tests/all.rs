use origami_engine::comp;

#[test]
fn should_work_with_expr() {
    let expr = "foo_bar";
    let expr = &expr;
    comp! {
        baz =>
        div {
            **expr;
        }
    }
    let html = baz!();
    assert_eq!(html.0, "<div>foo_bar</div>");
}

#[test]
fn should_be_self_closing() {
    comp! {
        component =>
        input;
    }
    let html = component!();
    assert_eq!(html.0, "<input/>");
}

#[test]
fn should_order_attributes_correctly() {
    comp! {
        component =>
        div hello="world" abc="def" hello abc="xyz" {}

    }
    let html = component!();
    assert_eq!(html.0, "<div hello abc=\"xyz\"></div>");
}

#[test]
fn should_order_attributes_correctly_when_using_placeholder() {
    comp! {
        component =>
        div hello="world" abc="def" $attr {}

    }
    let html = component!(attr {
        hello abc="xyz"
    });
    assert_eq!(html.0, "<div hello abc=\"xyz\"></div>");
}

#[test]
fn should_work_with_multiple_nested_components() {
    comp! {
        foo =>
        div {
            "foo_component"
        }
    }
    comp! {
        bar =>
        div {
            "bar_component"
            @foo!();
        }
    }
    comp! {
        baz =>
        div {
            "baz_component"
            @bar!();
        }
    }
    let html = baz!();
    assert_eq!(
        html.0,
        "<div>baz_component<div>bar_component<div>foo_component</div></div></div>"
    );
}

#[test]
fn should_work_with_conditionals() {
    comp! {
        foo =>
        div {
            if $bar == "bar"; {
                "bar_component"
            } else if $baz == "baz"; {
                "baz_component"
            } else {
                "foo_component"
            }
        }
    }
    let html = foo!(bar { "bar" }, baz { "not_baz" });
    let html2 = foo!(bar { "not_bar" }, baz { "baz" });
    let html3 = foo!(bar { "not_bar" }, baz { "not_baz" });
    assert_eq!(html.0, "<div>bar_component</div>");
    assert_eq!(html2.0, "<div>baz_component</div>");
    assert_eq!(html3.0, "<div>foo_component</div>");
}

#[test]
fn should_work_with_loops() {
    struct Points {
        x: i32,
        y: i32,
    }
    let points = [
        Points { x: 1, y: 2 },
        Points { x: 3, y: 4 },
        Points { x: 5, y: 6 },
    ];
    comp! {
        foo =>
        div {
            for point in $points; {
                div {
                    *point.x.to_string().as_str();
                    ","
                    *point.y.to_string().as_str();
                }
            }
        }
    }
    let html = foo!(points { points });
    assert_eq!(
        html.0,
        "<div><div>1,2</div><div>3,4</div><div>5,6</div></div>"
    );
}

#[test]
fn should_work_with_match_expression() {
    comp! {
        component =>
        div {
            match $value; {
                "bar" => {
                     "bar_component"
                },
                "baz" => {
                     "baz_component"
                },
                _ => {
                     "foo_component"
                },
            }
        }
    }
    let html = component!(value { "bar" });
    let html2 = component!(value { "baz" });
    let html3 = component!(value { "" });
    assert_eq!(html.0, "<div>bar_component</div>");
    assert_eq!(html2.0, "<div>baz_component</div>");
    assert_eq!(html3.0, "<div>foo_component</div>");
}

#[cfg(feature = "html_escape")]
#[test]
fn should_escape() {
    comp! {
        component =>
        div {
            "<div>foo_bar</div>"
        }
    }
    let html = component!();
    assert_eq!(html.0, "<div>&lt;div&gt;foo_bar&lt;/div&gt;</div>");
}

#[cfg(feature = "html_escape")]
#[test]
fn should_inherit_parent_escape_state() {
    comp! {
        component =>
        div noescape {
            "<div>foo_bar</div>"
            div {
                "<div>foo_bar</div>"
            }
        }
    }
    let html = component!();
    assert_eq!(
        html.0,
        "<div><div>foo_bar</div><div><div>foo_bar</div></div></div>"
    );
}

#[cfg(feature = "html_escape")]
#[test]
fn should_escape_inner() {
    comp! {
        component =>
        div noescape {
            "<div>foo_bar</div>"
            div escape {
                "<div>foo_bar</div>"
            }
        }
    }
    let html = component!();
    assert_eq!(
        html.0,
        "<div><div>foo_bar</div><div>&lt;div&gt;foo_bar&lt;/div&gt;</div></div>"
    );
}

#[cfg(feature = "html_escape")]
#[test]
fn should_not_escape_expr() {
    let expr = "<div>foo_bar</div>";
    comp! {
        component =>
        div {
            *expr;!
        }
    }
    let html = component!();
    assert_eq!(html.0, "<div><div>foo_bar</div></div>");
}

#[cfg(feature = "html_escape")]
#[test]
fn should_not_escape_literal() {
    comp! {
        component =>
        div {
            "<div>foo_bar</div>"!
        }
    }
    let html = component!();
    assert_eq!(html.0, "<div><div>foo_bar</div></div>");
}

#[cfg(feature = "html_escape")]
#[test]
fn should_not_escape_inner_comp() {
    comp! {
        bar =>
        div {
            "<div>foo_bar</div>"
        }
    }
    comp! {
        foo =>
        div {
            @bar!();!
        }
    }
    let html = foo!();
    assert_eq!(html.0, "<div><div><div>foo_bar</div></div></div>");
}

#[cfg(not(feature = "html_escape"))]
#[test]
fn should_not_escape() {
    comp! {
        component =>
        div {
            "<div>foo_bar</div>"
        }
    }
    let html = component!();
    assert_eq!(html.0, "<div><div>foo_bar</div></div>");
}

#[cfg(feature = "minify_html")]
#[test]
fn should_minify_js() {
    comp! {
        component =>
        script {
            r#"function foo() {
                return "hello world";
            }"#
        }
    }
    let html = component!();
    assert_eq!(
        html.0,
        "<script>function foo() { return \"hello world\"; }</script>"
    );
}

#[cfg(feature = "minify_html")]
#[test]
fn should_not_minify_js() {
    comp! {
        component =>
        script nominify {
            r#"function foo() {
                return "hello world";
            }"#
        }
    }
    let html = component!();
    assert_eq!(
        html.0,
        r#"<script>function foo() {
                return "hello world";
            }</script>"#
    );
}

#[cfg(feature = "minify_html")]
#[test]
fn should_minify_style() {
    comp! {
        component =>
        style {
            r#"
                body {
                    background-color: red;
                }
            "#
        }
    }
    let html = component!();
    assert_eq!(html.0, "<style>body { background-color: red; }</style>");
}

#[cfg(feature = "minify_html")]
#[test]
fn should_not_minify_style() {
    comp! {
        component =>
        style nominify {
            r#"
                body {
                    background-color: red;
                }
            "#
        }
    }
    let html = component!();
    assert_eq!(
        html.0,
        r#"<style>
                body {
                    background-color: red;
                }
            </style>"#
    );
}

#[cfg(feature = "minify_html")]
#[test]
fn should_move_script_when_minify_html_is_enabled() {
    comp! {
        bar =>
        div {
            "bar_component"
            script script_name="bar_script" {
                r#"function foo() {
                    return "hello world";
                }"#
            }
        }
    }
    comp! {
        foo =>
        div {
            "foo_component"
            @bar!();
        }
        script_use bar_script;
    }
    let html = foo!();
    assert_eq!(
        html.0,
        "<div>foo_component<div>bar_component</div></div><script>function foo() { return \"hello world\"; }</script>"
    );
}

#[cfg(not(feature = "minify_html"))]
#[test]
fn should_move_script_when_minify_html_is_not_enabled() {
    comp! {
        bar =>
        div {
            "bar_component"
            script script_name="bar_script" {
                r#"function foo() {
                    return "hello world";
                }"#
            }
        }
    }
    comp! {
        foo =>
        div {
            "foo_component"
            @bar!();
        }
        script_use bar_script;
    }
    let html = foo!();
    assert_eq!(
        html.0,
        r#"<div>foo_component<div>bar_component</div></div><script>function foo() {
                    return "hello world";
                }</script>"#
    );
}
