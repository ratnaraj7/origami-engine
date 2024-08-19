pub use origami_macros::og;

pub trait Layout<T> {
    fn extend(&self, s: &mut String, blocks: &T);
    fn extend_with_childrens(&self, s: &mut String, blocks: &T, childrens: impl Fn(&mut String));
}

pub trait Origami {
    fn to_html(&self) -> String;
    fn push_html_to_string(&self, s: &mut String);
    fn push_html_to_string_with_childrens(&self, s: &mut String, childrens: impl Fn(&mut String));
}
