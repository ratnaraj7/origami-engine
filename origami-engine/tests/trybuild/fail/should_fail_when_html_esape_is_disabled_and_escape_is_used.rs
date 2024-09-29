use origami_engine::comp;

fn main() {
    comp! {
        foo =>
        div escape {

        }
    }
    foo!();
}
