use origami_engine::comp;
fn main() {
    comp! {
        foo =>
        script noescape {

        }
    }
    foo!();
}
