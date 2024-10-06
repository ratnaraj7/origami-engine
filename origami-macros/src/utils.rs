pub mod kw {
    use syn::custom_keyword;
    custom_keyword!(nominify);
    custom_keyword!(script_use);
    custom_keyword!(size_hint);
    custom_keyword!(escape);
    custom_keyword!(noescape);
    custom_keyword!(literals);
    custom_keyword!(call);
}

macro_rules! bail {
    ($input:expr, $msg:expr) => {
        return Err(::syn::Error::new($input.span(), $msg))
    };
}
pub(crate) use bail;

macro_rules! combine_to_lit {
    ($($input:expr),*) => {{
        combine_to_lit!(@internal span => ::proc_macro2::Span::call_site(), $($input),*)
    }};
    ($span:expr => $($input:expr),*) => {{
        combine_to_lit!(@internal span => $span, $($input),*)
    }};
    (@internal span => $span:expr, $($input:expr),*) => {{
        let mut s = String::new();
        $(s.push_str(&$input);)*
        ::syn::LitStr::new(&s, $span)
    }}
}
pub(crate) use combine_to_lit;

#[cfg(test)]
mod tests {
    use proc_macro2::Span;
    use syn::{Ident, LitStr};

    #[test]
    fn combine_lit() {
        let lit = LitStr::new("test", Span::call_site());
        let ident = Ident::new("test", Span::call_site());
        assert_eq!(
            combine_to_lit!(lit.span() => "foo", "bar", lit.value(), ident.to_string()),
            LitStr::new("foobartesttest", Span::call_site())
        );
    }
}
