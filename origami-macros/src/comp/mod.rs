use indexmap::IndexSet;
use proc_macro2::{Delimiter, TokenTree};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::token::Paren;
use syn::{parenthesized, Ident, Token, Visibility};

use crate::utils::bail;

pub struct Component {
    vis: Visibility,
    name: syn::Ident,
    ts: proc_macro2::TokenStream,
    props: IndexSet<Ident>,
}

impl Parse for Component {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis = input.parse()?;
        let name = input.parse()?;
        let mut props = IndexSet::new();
        if input.peek(Paren) {
            let content;
            parenthesized!(content in input);
            let mut count = 0;
            while !content.is_empty() {
                if count > 0 {
                    if content.peek(Token![,]) {
                        content.parse::<Token![,]>()?;
                        if content.is_empty() {
                            break;
                        }
                    } else {
                        bail!(content, "Expected `,`");
                    }
                }
                let prop: Ident = content.parse()?;
                if props.contains(&prop) {
                    bail!(prop, "Duplicate prop: `{}`");
                }
                props.insert(prop);
                count += 1;
            }
        }
        input.parse::<Token![=>]>()?;
        let ts = macro_rep(input, &props)?;
        Ok(Component {
            vis,
            name,
            ts,
            props,
        })
    }
}

impl ToTokens for Component {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let props = self.props.iter();
        let front_comma_props = quote! {
            #(, #props {$($#props:tt)*})*
        };
        let props = self.props.iter();
        let back_comma_props = quote! {
            #(#props {$($#props:tt)*}),*
        };
        let name = &self.name;
        let vis = &self.vis;
        let vis_t = {
            if vis == &syn::Visibility::Inherited {
                quote! {}
            } else {
                quote! { #vis use #name; }
            }
        };
        let ts = &self.ts;
        tokens.extend(quote! {
            macro_rules! #name {
              (@component escape { $($escape:tt)* }, internal { $($internal:tt)* }, #back_comma_props) => {
                  ::origami_engine::anon! {
                      $($internal)*,
                      childrens $($escape)* {
                          #ts
                      }
                  }
              };
              (#back_comma_props) => {{
                  let mut s = String::new();
                  ::origami_engine::anon! {
                      childrens {
                          #ts
                      }
                  }
                  ::origami_engine::Origami(s)
              }};
              (cap => $capacity:expr #front_comma_props) => {{
                  let mut s = String::with_capacity($capacity);
                  ::origami_engine::anon! {
                      childrens {
                          #ts
                      }
                  }
                  ::origami_engine::Origami(s)
              }};
            }
            #vis_t
        });
    }
}

#[derive(Debug)]
enum Next {
    Ident,
    Any,
}

fn macro_rep(ps: ParseStream, props: &IndexSet<Ident>) -> syn::Result<proc_macro2::TokenStream> {
    let mut ts = proc_macro2::TokenStream::new();
    while !ps.is_empty() {
        ts.extend(ps.parse::<TokenTree>()?.into_token_stream());
    }
    let mut next = Next::Any;
    let ts = handle_token(&mut next, ts, props)?;
    Ok(ts)
}

fn handle_token(
    next: &mut Next,
    o_ts: proc_macro2::TokenStream,
    props: &IndexSet<Ident>,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut ts = proc_macro2::TokenStream::new();
    let mut o_ts_i = o_ts.into_iter();
    while let Some(token) = o_ts_i.next() {
        match (&next, &token) {
            (Next::Any, TokenTree::Group(group)) => {
                if let Delimiter::Brace = group.delimiter() {
                    let rts = handle_token(next, group.stream(), props)?;
                    ts.extend(quote! {
                        {
                            #rts
                        }
                    });
                    continue;
                } else {
                    ts.extend(token.into_token_stream());
                }
            }
            (Next::Any, TokenTree::Punct(p)) if p.as_char() == '@' => {
                *next = Next::Ident;
            }
            (Next::Ident, _) => {
                let n_token = o_ts_i.next();
                match (&token, &n_token) {
                    (TokenTree::Ident(ident), Some(TokenTree::Punct(n_t)))
                        if props.contains(ident) && n_t.as_char() == ';' =>
                    {
                        ts.extend(quote! {
                            $($#ident)*
                        });
                    }
                    _ => ts.extend(quote! {
                        @#token #n_token
                    }),
                }
                *next = Next::Any;
            }
            _ => {
                ts.extend(token.into_token_stream());
            }
        }
    }
    Ok(ts)
}
