use indexmap::IndexMap;
use syn::parse::{Parse, ParseStream};
use syn::token::Brace;
use syn::{Expr, Ident, LitStr, Token};

use crate::utils::bail;
use crate::utils::kw::{escape, noescape, nominify};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum AttributeKey {
    LitStr(LitStr),
    Ident(Ident),
    #[cfg(feature = "html_escape")]
    Escape,
    #[cfg(feature = "html_escape")]
    NoEscape,
    #[cfg(feature = "minify_html")]
    NoMinify,
}

impl Parse for AttributeKey {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(escape) {
            if cfg!(not(feature = "html_escape")) {
                bail!(input, "Enable `html_escape` feature to use `\"escape\"`.")
            }
            input.parse::<escape>()?;
            #[cfg(feature = "html_escape")]
            return Ok(Self::Escape);
        }
        if input.peek(noescape) {
            if cfg!(not(feature = "html_escape")) {
                bail!(input, "Enable `html_escape` feature to use `\"noescape\"`.")
            }
            input.parse::<noescape>()?;
            #[cfg(feature = "html_escape")]
            return Ok(Self::NoEscape);
        }
        if input.peek(nominify) {
            if cfg!(not(feature = "minify_html")) {
                bail!(
                    input,
                    "It is not possible to use `\"nominify\"` without `minify_html` feature."
                )
            }
            input.parse::<nominify>()?;
            #[cfg(feature = "minify_html")]
            return Ok(Self::NoMinify);
        }
        if input.peek(Ident) {
            return Ok(Self::Ident(input.parse()?));
        }
        if input.peek(LitStr) {
            return Ok(Self::LitStr(input.parse()?));
        }
        bail!(input, "Expected an attribute key.")
    }
}

#[derive(Debug)]
pub enum AttributeValue {
    LitStr(LitStr),
    Expr(Expr),
}

impl Parse for AttributeValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            return Ok(Self::LitStr(input.parse()?));
        }
        if input.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            return Ok(Self::Expr(input.parse()?));
        }
        bail!(input, "Expected string or expression.")
    }
}

#[derive(Debug)]
pub struct Attributes(pub IndexMap<AttributeKey, Option<AttributeValue>>);

impl Parse for Attributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut map = IndexMap::new();
        while !input.peek(Brace) && !input.peek(Token![;]) {
            let key = input.parse::<AttributeKey>()?;
            let value = match key {
                #[cfg(feature = "html_escape")]
                AttributeKey::Escape | AttributeKey::NoEscape => None,
                _ => {
                    if input.peek(Token![=]) {
                        input.parse::<Token![=]>()?;
                        Some(input.parse::<AttributeValue>()?)
                    } else {
                        None
                    }
                }
            };
            map.insert(key.clone(), value);
        }
        Ok(Self(map))
    }
}
