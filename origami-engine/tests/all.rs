use origami_engine::{og, Origami};

#[test]
fn should_have_attributes_in_order() {
    struct Foo {
        baz: String,
    }
    og! {
        Foo =>
        div class="foo" "hx-get"="/foo" bar baz=(self.baz.as_str()) bar {

        }
    }
    let html = Foo {
        baz: "baz".to_string(),
    }
    .to_html();
    assert_eq!(
        html,
        "<div class=\"foo\" hx-get=\"/foo\" bar baz=\"baz\"></div>"
    );
}

#[test]
fn should_consolidate_attributes_in_order_with_value() {
    struct Foo {
        baz: String,
    }
    og! {
        Foo =>
        div class="foo" "hx-get"="/foo" bar baz=(self.baz.as_str()) bar="bar" {

        }
    }
    let html = Foo {
        baz: "baz".to_string(),
    }
    .to_html();
    assert_eq!(
        html,
        "<div class=\"foo\" hx-get=\"/foo\" bar=\"bar\" baz=\"baz\"></div>"
    );
}

#[test]
fn should_consolidate_attributes_in_order_with_value_2() {
    struct Foo {
        baz: String,
    }
    og! {
        Foo =>
        div class="foo" "hx-get"="/foo" bar="baz" baz=(self.baz.as_str()) bar="bar" {

        }
    }
    let html = Foo {
        baz: "baz".to_string(),
    }
    .to_html();
    assert_eq!(
        html,
        "<div class=\"foo\" hx-get=\"/foo\" bar=\"bar\" baz=\"baz\"></div>"
    );
}

#[cfg(feature = "html_escape")]
#[test]
fn should_work_with_component() {
    struct Bar;
    og! {
        Bar =>
        div class="bar" "hx-get"="/bar" {
            "bar"
        }
    }
    struct Foo;
    og! {
        Foo =>
        div class="foo" "hx-get"="/foo" {
            @Bar{};
        }
    }
    let html = Foo.to_html();
    assert_eq!(
        html,
        "<div class=\"foo\" hx-get=\"/foo\"><div class=\"bar\" hx-get=\"/bar\">bar</div></div>"
    );
}

#[cfg(feature = "html_escape")]
#[test]
fn should_iterate_components() {
    #[derive(Clone)]
    struct Bar;
    og! {
        Bar =>
        div class="bar" "hx-get"="/bar" {
            "bar"
        }
    }
    struct Foo {
        bars: Vec<Bar>,
    }
    og! {
        Foo =>
        div class="foo" "hx-get"="/foo" {
            @(..self.bars.iter());
        }
    }
    let html = Foo { bars: vec![Bar; 3] }.to_html();
    assert_eq!(
        html,
        "<div class=\"foo\" hx-get=\"/foo\"><div class=\"bar\" hx-get=\"/bar\">bar</div><div class=\"bar\" hx-get=\"/bar\">bar</div><div class=\"bar\" hx-get=\"/bar\">bar</div></div>"
    );
}

#[test]
#[cfg(feature = "html_escape")]
fn should_escape_lit() {
    struct Foo;
    og! {
        Foo =>
        div class="foo" "hx-get"="/foo" {
            "<div class=\"foo\" hx-get=\"/foo\">foo</div>"
        }
    }
    let html = Foo.to_html();
    assert_eq!(html, "<div class=\"foo\" hx-get=\"/foo\">&lt;div class=\"foo\" hx-get=\"/foo\"&gt;foo&lt;/div&gt;</div>");
}

#[cfg(not(feature = "html_escape"))]
#[test]
fn should_not_escape_lit() {
    struct Foo;
    og! {
        Foo =>
        div class="foo" "hx-get"="/foo" {
            "<div class=\"foo\" hx-get=\"/foo\">foo</div>"
        }
    }

    let html = Foo.to_html();
    assert_eq!(
        html,
        "<div class=\"foo\" hx-get=\"/foo\"><div class=\"foo\" hx-get=\"/foo\">foo</div></div>"
    );
}

#[cfg(feature = "html_escape")]
#[test]
fn should_not_escape_anything_inside_noescape_block() {
    struct Foo;
    og! {
        Foo =>
        div class="foo" "hx-get"="/foo" {
            div noescape class="foo" "hx-get"="/foo" {
                "<div class=\"foo\" hx-get=\"/foo\">foo</div>"
            }
        }
    }
    let html = Foo.to_html();
    assert_eq!(
        html,
        "<div class=\"foo\" hx-get=\"/foo\"><div class=\"foo\" hx-get=\"/foo\"><div class=\"foo\" hx-get=\"/foo\">foo</div></div></div>"
    );
}

#[cfg(feature = "html_escape")]
#[test]
fn should_not_escape_anything_inside_noescape_block_and_escape_everything_inside_escape_block() {
    struct Foo;
    og! {
        Foo =>
        div class="foo" "hx-get"="/foo" {
            div noescape class="foo" "hx-get"="/foo" {
                "<div class=\"foo\" hx-get=\"/foo\">foo</div>"
                "<>hello"
                div escape class="foo" "hx-get"="/foo" {
                    "<div class=\"foo\" hx-get=\"/foo\">foo</div>"
                }
            }
        }
    }
    let html = Foo.to_html();
    assert_eq!(html,
        "<div class=\"foo\" hx-get=\"/foo\"><div class=\"foo\" hx-get=\"/foo\"><div class=\"foo\" hx-get=\"/foo\">foo</div><>hello<div class=\"foo\" hx-get=\"/foo\">&lt;div class=\"foo\" hx-get=\"/foo\"&gt;foo&lt;/div&gt;</div></div></div>");
}
