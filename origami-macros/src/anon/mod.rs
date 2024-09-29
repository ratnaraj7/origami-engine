use quote::{quote, ToTokens};
use syn::parse::Parse;
use syn::{parse_quote, Expr, Token};

mod children;

#[cfg(feature = "html_escape")]
use crate::utils::kw::{escape, noescape};

use self::children::{Children, ChildrensE, Context};

pub struct Anon {
    childrens: ChildrensE,
}

impl Parse for Anon {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        #[cfg(feature = "html_escape")]
        let escape = if input.peek(escape) {
            input.parse::<escape>()?;
            input.parse::<Token![,]>()?;
            true
        } else if input.peek(noescape) {
            input.parse::<noescape>()?;
            input.parse::<Token![,]>()?;
            false
        } else {
            true
        };
        let expr = if input.peek2(Token![,]) {
            let expr = input.parse::<Expr>()?;
            input.parse::<Token![,]>()?;
            expr
        } else {
            parse_quote! {
                s
            }
        };
        let mut childrens = Vec::new();
        let mut ctx = Context {
            #[cfg(feature = "html_escape")]
            escape,
        };
        while !input.is_empty() {
            childrens.push(Children::parse(input, &mut ctx)?);
        }
        Ok(Anon {
            childrens: ChildrensE { s: expr, childrens },
        })
    }
}

impl ToTokens for Anon {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let childrens = &self.childrens;
        tokens.extend(quote! {
            #childrens
        })
    }
}
