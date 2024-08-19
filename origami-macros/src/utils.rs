pub mod kw {
    use syn::custom_keyword;
    custom_keyword!(block);
    custom_keyword!(extend);
    custom_keyword!(layout);
    custom_keyword!(size_hint);
    custom_keyword!(childrens);
    custom_keyword!(include);
    custom_keyword!(escape);
    custom_keyword!(noescape);
}

macro_rules! bail {
    ($input:expr, $msg:expr) => {
        return Err(::syn::Error::new($input.span(), $msg))
    };
}
pub(crate) use bail;
