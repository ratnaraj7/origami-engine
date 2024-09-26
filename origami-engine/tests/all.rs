use origami_engine::comp;
use origami_macros::anon;

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
            if $bar == "bar" {
                "bar_component"
            } else if $baz == "baz" {
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
