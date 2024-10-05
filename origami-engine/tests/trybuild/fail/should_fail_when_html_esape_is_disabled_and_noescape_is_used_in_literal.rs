use origami_engine::comp;

fn main() {
    comp! {
        foo =>
        div {
            "<div></div>";!
        }
    }
    foo!();
}
