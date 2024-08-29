use indexmap::IndexMap;
use proc_macro2::Span;
use quote::{quote, quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::token::{Brace, Paren};
use syn::{parenthesized, Expr, ExprField, Ident, LitStr, Token};

use crate::utils::kw::{escape, noescape};
use crate::utils::{bail, combine_to_lit};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum AttributeKey {
    LitStr(LitStr),
    Ident(Ident),
    Iter(ExprField),
    #[cfg(feature = "html_escape")]
    Escape,
    #[cfg(feature = "html_escape")]
    NoEscape,
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
        if input.peek(Ident) {
            return Ok(Self::Ident(input.parse()?));
        }
        if input.peek(LitStr) {
            return Ok(Self::LitStr(input.parse()?));
        }
        if input.peek(Token![..]) {
            input.parse::<Token![..]>()?;
            if !input.peek(Token![self]) {
                bail!(input, "Expected `self`.")
            }
            let expr: ExprField = input.parse()?;
            return Ok(Self::Iter(expr));
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
        if input.peek(Paren) {
            let content;
            parenthesized!(content in input);
            return Ok(Self::Expr(content.parse()?));
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
                AttributeKey::Iter(_) => None,
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

enum AttributeType {
    Lit(LitStr),
    TokenStream(proc_macro2::TokenStream),
}

impl ToTokens for AttributeType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Lit(lit) => tokens.extend(quote! {s.push_str(#lit);}),
            Self::TokenStream(ts) => ts.to_tokens(tokens),
        }
    }
}

impl ToTokens for Attributes {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut attribute_types = Vec::new();
        for (k, v) in &self.0 {
            let last = attribute_types.last_mut();
            match k {
                AttributeKey::Ident(ident) => {
                    if let Some(AttributeType::Lit(lit)) = last {
                        let new_lit =
                            combine_to_lit!(ident.span() => lit.value(), " ", ident.to_string());
                        *lit = new_lit;
                    } else {
                        let lit = combine_to_lit!(ident.span() => " ", ident.to_string());
                        attribute_types.push(AttributeType::Lit(lit));
                    }
                }
                AttributeKey::LitStr(literal) => {
                    if let Some(AttributeType::Lit(lit)) = last {
                        let new_lit =
                            combine_to_lit!(literal.span() => lit.value(), " ", literal.value());
                        *lit = new_lit;
                    } else {
                        let lit = combine_to_lit!(literal.span() => " ", literal.value());
                        attribute_types.push(AttributeType::Lit(lit));
                    }
                }
                AttributeKey::Iter(expr) => {
                    attribute_types.push(AttributeType::TokenStream(quote_spanned! {
                        expr.span() =>
                        for (key, val) in #expr.iter() {
                            s.push(' ');
                            s.push_str(key);
                            s.push_str("=\"");
                            s.push_str(val);
                            s.push('"');
                        }
                    }))
                }
                #[cfg(feature = "html_escape")]
                AttributeKey::Escape | AttributeKey::NoEscape => {}
            }
            let last = attribute_types.last_mut();
            if let Some(v) = v {
                match v {
                    AttributeValue::LitStr(literal) => {
                        if let Some(AttributeType::Lit(lit)) = last {
                            let new_lit = combine_to_lit!(literal.span() => lit.value(), "=\"", literal.value(), "\"");
                            *lit = new_lit;
                        } else {
                            let lit =
                                combine_to_lit!(literal.span() => "=\"", literal.value(), "\"");
                            attribute_types.push(AttributeType::Lit(lit));
                        }
                    }
                    AttributeValue::Expr(expr) => {
                        if let Some(AttributeType::Lit(lit)) = last {
                            let new_lit = combine_to_lit!(lit.value(), "=\"");
                            *lit = new_lit;
                        } else {
                            attribute_types
                                .push(AttributeType::Lit(LitStr::new("=\"", expr.span())));
                        }
                        attribute_types.push(AttributeType::TokenStream(quote_spanned! {
                            expr.span() =>
                            s.push_str(#expr);
                        }));
                        attribute_types.push(AttributeType::TokenStream(quote! {
                            s.push('"');
                        }));
                    }
                }
            }
        }
        tokens.extend(quote! {
            #(#attribute_types)*
        });
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[derive(Debug)]
    struct TestAttributes(Attributes);

    impl Parse for TestAttributes {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let attrs: Attributes = input.parse()?;
            input.parse::<Token![;]>()?;
            Ok(Self(attrs))
        }
    }

    #[cfg(feature = "html_escape")]
    #[test]
    fn should_work() {
        let TestAttributes(attrs) = parse_quote! {
            class="test" escape style="test" id=(self.id) ..self.k noescape class="test2" bar;
        };
        let tokens = quote! {
            #attrs
        };
        assert_eq!(
            tokens.to_string(),
            quote! {
                s.push_str(" class=\"test2\" style=\"test\" id=\"");
                s.push_str(self.id);
                s.push('"');
                for (key, val) in self.k.iter() {
                    s.push(' ');
                    s.push_str(key);
                    s.push_str("=\"");
                    s.push_str(val);
                    s.push('"');
                }
                s.push_str(" bar");
            }
            .to_string()
        );
    }

    #[cfg(not(feature = "html_escape"))]
    #[test]
    #[should_panic(expected = "Enable `html_escape` feature to use `\"escape\"`.")]
    fn should_panic_if_escape_is_used_when_feature_is_disabled() {
        let TestAttributes(attrs) = parse_quote! {
            class="test" escape;
        };
    }

    #[cfg(not(feature = "html_escape"))]
    #[test]
    #[should_panic(expected = "Enable `html_escape` feature to use `\"noescape\"`.")]
    fn should_panic_if_noescape_is_used_when_feature_is_disabled() {
        let TestAttributes(attrs) = parse_quote! {
            class="test" noescape;
        };
    }
}
