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
            *expr;!
        }
    }
    comp! {
        expression_with_escape =>
        div {
            *expr;
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

criterion_group!(benches, bench_escape, bench_minify,);
criterion_main!(benches);
