use origami_engine::comp;

fn main() {
    comp! {
        bar =>
        div {

        }
    }

    comp! {
        foo =>
        @bar!();!
    }
    foo!();
}
