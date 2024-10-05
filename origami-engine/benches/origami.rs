use criterion::{black_box, criterion_group, criterion_main, Criterion};
use origami_engine::comp;

fn bench_escape(c: &mut Criterion) {
    comp! {
        literal_without_escape =>
        div {
            "<div>foo</div>"!
        }
    }
    comp! {
        literal_with_escape =>
        div {
            "<div>foo</div>"
        }
    }
    let expr = "<div>foo</div>";
    comp! {
        expression_without_escape =>
        div {
            @expr;!
        }
    }
    comp! {
        expression_with_escape =>
        div {
            @expr;
        }
    }
    c.bench_function("literal without escape", |b| {
        b.iter(|| black_box(literal_without_escape!(cap => 50)))
    });
    c.bench_function("literal with escape", |b| {
        b.iter(|| black_box(literal_with_escape!(cap => 50)))
    });
    c.bench_function("expression without escape", |b| {
        b.iter(|| black_box(expression_without_escape!(cap => 50)))
    });
    c.bench_function("expression with escape", |b| {
        b.iter(|| black_box(expression_with_escape!(cap => 50)))
    });
}

fn bench_minify(c: &mut Criterion) {
    comp! {
        literal_without_minify =>
        script nominify {
            "function foo() { return \"hello world\"; }"
        }
    }
    comp! {
        literal_with_minify =>
        script {
            "function foo() { return \"hello world\"; }"
        }
    }
    let expr = "function foo() { return \"hello world\"; }";
    comp! {
        expression_without_minify =>
        script nominify {
            expr
        }
    }
    comp! {
        expression_with_minify =>
        script {
            expr
        }
    }
    c.bench_function("literal without minify", |b| {
        b.iter(|| black_box(literal_without_minify!(cap => 100)))
    });
    c.bench_function("literal with minify", |b| {
        b.iter(|| black_box(literal_with_minify!(cap => 100)))
    });
    c.bench_function("expression without minify", |b| {
        b.iter(|| black_box(expression_without_minify!(cap => 100)))
    });
    c.bench_function("expression with minify", |b| {
        b.iter(|| black_box(expression_with_minify!(cap => 100)))
    });
}

fn bench_full_page(c: &mut Criterion) {
    comp! {
        button_component(attr, label) =>
        button class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600" @attr; {
            @label;
        }
    }
    comp! {
        layout_component(content) =>
        nav {
            ul {
                li { a { "Home" } }
                li { a { "About" } }
                li { a { "Contact" } }
            }
        }
        main {
            @content;
        }
        footer {
            p { "© 2024 Your Company" }
        }
        script script_name="layout_script" {
            r#"console.log('layout script loaded!');"#
        }
    }
    let show_extra_content = true;
    let items = ["Item 1", "Item 2", "Item 3"];
    let status = "success";
    comp! {
        home =>
        call layout_component {
            content {
                h1 { "Welcome to the Homepage!" }
                p { "This is the main content of the homepage." }
                call button_component { attr { onclick="alert('clicked')" }, label { "Click Me" } }
                if show_extra_content; {
                    p { "Here is some extra content that is conditionally rendered." }
                }
                h2 { "List of Items:" }
                ul {
                    for item in items.iter(); {
                        li { @item; }
                    }
                }
                match status; {
                    "success" => {
                        p { "Operation was successful!" }
                    },
                    "error" => {
                        p { "There was an error." }
                    },
                    _ => {
                        p { "Unknown status." }
                    }
                }
                p escape {
                    "<div>This will be escaped: <strong>Important!</strong></div>"
                }
                p noescape {
                    "<div>This will not be escaped: <strong>Unsafe HTML</strong></div>"
                }
            }
        }
        script_use layout_script;
    }
    c.bench_function("full page", |b| b.iter(|| black_box(home!(cap => 800))));
}

criterion_group!(benches, bench_escape, bench_minify, bench_full_page);
criterion_main!(benches);
