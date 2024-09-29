use origami_engine::comp;
fn main() {
    comp! {
        foo =>
        style noescape {

        }
    }
    foo!();
}
