use origami_engine::comp;

fn main() {
    let expr = "<div></div>";
    comp! {
        foo =>
        div {
            @expr;!
        }
    }
    foo!();
}
