use std::collections::HashMap;

use quote::{quote, quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::token::{Brace, Paren};
use syn::{parenthesized, parse_quote, Expr, ExprField, Ident, LitChar, LitStr, Token};

use crate::utils::bail;
use crate::utils::kw::{escape, noescape};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum AttributeKey {
    LitStr(LitStr),
    LitChar(LitChar),
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
        if input.peek(LitChar) {
            return Ok(Self::LitChar(input.parse()?));
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
    LitChar(LitChar),
    Expr(Expr),
}

impl Parse for AttributeValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            return Ok(Self::LitStr(input.parse()?));
        }
        if input.peek(LitChar) {
            return Ok(Self::LitChar(input.parse()?));
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
pub struct Attributes(
    pub HashMap<AttributeKey, Option<AttributeValue>>,
    // for key ordering
    Vec<AttributeKey>,
);

impl Parse for Attributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut map = HashMap::new();
        let mut position_map = HashMap::new();
        let mut count = 0;
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
            position_map.insert(key, count);
            count += 1;
        }
        let mut positions: Vec<_> = position_map.into_iter().collect();
        positions.sort_by(|a, b| a.1.cmp(&b.1));
        let positions: Vec<_> = positions.into_iter().map(|(key, _)| key).collect();
        Ok(Self(map, positions))
    }
}

impl ToTokens for Attributes {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for k in &self.1 {
            let v = match self.0.get(k) {
                Some(v) => v,
                None => unreachable!(),
            };
            match k {
                AttributeKey::Ident(ident) => {
                    tokens.extend(quote! {
                        s.push(' ');
                    });
                    let literal = ident.to_string();
                    let literal: LitStr = parse_quote! {
                        #literal
                    };
                    tokens.extend(quote! {
                        s.push_str(#literal);
                    });
                }
                AttributeKey::LitStr(literal) => {
                    tokens.extend(quote! {
                        s.push(' ');
                    });
                    tokens.extend(quote! {
                        s.push_str(#literal);
                    });
                }
                AttributeKey::LitChar(literal) => {
                    tokens.extend(quote! {
                        s.push(' ');
                    });
                    tokens.extend(quote! {
                        s.push(#literal);
                    });
                }
                AttributeKey::Iter(expr) => tokens.extend(quote_spanned! {
                    expr.span() =>
                    for (key, val) in #expr.iter() {
                        s.push(' ');
                        s.push_str(key);
                        s.push_str("=\"");
                        s.push_str(val);
                        s.push('"');
                    }
                }),
                #[cfg(feature = "html_escape")]
                AttributeKey::Escape | AttributeKey::NoEscape => {}
            }
            if let Some(v) = v {
                tokens.extend(quote! {
                    s.push_str("=\"");
                });
                match v {
                    AttributeValue::LitStr(literal) => {
                        tokens.extend(quote! {
                            s.push_str(#literal);
                        });
                    }
                    AttributeValue::LitChar(literal) => {
                        tokens.extend(quote! {
                            s.push(#literal);
                        });
                    }
                    AttributeValue::Expr(expr) => {
                        tokens.extend(quote_spanned! {
                            expr.span() =>
                            s.push_str(#expr);
                        });
                    }
                }
                tokens.extend(quote! {
                    s.push('"');
                });
            }
        }
    }
}
