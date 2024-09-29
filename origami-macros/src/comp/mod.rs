use proc_macro2::{Delimiter, TokenTree};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::Token;

use crate::utils::bail;

pub struct Component {
    name: syn::Ident,
    ts: proc_macro2::TokenStream,
    vars: Vec<syn::Ident>,
}

impl Parse for Component {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![=>]>()?;
        let (ts, vars) = macro_rep(input)?;
        Ok(Component { name, ts, vars })
    }
}

impl ToTokens for Component {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let vars = &self.vars;
        let name = &self.name;
        let ts = &self.ts;
        tokens.extend(quote! {
            macro_rules! #name {
              (#(#vars {$($#vars:tt)*}),*) => {{
                  let mut s = String::new();
                  ::origami_engine::anon! {
                      #ts
                  }
                  ::origami_engine::Origami(s)
              }};
              (noescape, #(#vars {$($#vars:tt)*}),*) => {{
                  let mut s = String::new();
                  ::origami_engine::anon! {
                      noescape,
                      #ts
                  }
                  ::origami_engine::Origami(s)
              }};
              (cap => $capacity:expr #(,#vars {$($#vars:tt)*})*) => {{
                  let mut s = String::with_capacity($capacity);
                  ::origami_engine::anon! {
                      #ts
                  }
                  ::origami_engine::Origami(s)
              }};
              (noescape, cap => $capacity:expr #(,#vars {$($#vars:tt)*})*) => {{
                  let mut s = String::with_capacity($capacity);
                  ::origami_engine::anon! {
                      noescape,
                      #ts
                  }
                  ::origami_engine::Origami(s)
              }};
              ($s:expr => #(#vars {$($#vars:tt)*}),*) => {
                  ::origami_engine::anon! {
                      $s,
                      #ts
                  }
              };
              (noescape, $s:expr => #(#vars {$($#vars:tt)*}),*) => {
                  ::origami_engine::anon! {
                      noescape,
                      $s,
                      #ts
                  }
              };
            }
        });
    }
}

enum Next {
    Ident,
    Any,
}

fn macro_rep(ps: ParseStream) -> syn::Result<(proc_macro2::TokenStream, Vec<syn::Ident>)> {
    let mut ts = proc_macro2::TokenStream::new();
    let mut vars = Vec::new();
    let mut next = Next::Any;
    while !ps.is_empty() {
        let token = ps.parse::<TokenTree>()?;
        ts.extend(handle_token(
            &mut next,
            token.into_token_stream(),
            &mut vars,
        )?);
    }
    Ok((ts, vars))
}

fn handle_token(
    next: &mut Next,
    o_ts: proc_macro2::TokenStream,
    vars: &mut Vec<syn::Ident>,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut ts = proc_macro2::TokenStream::new();
    for token in o_ts {
        match (&next, &token) {
            (Next::Any, TokenTree::Group(group)) => {
                if let Delimiter::Brace = group.delimiter() {
                    let rts = handle_token(next, group.stream(), vars)?;
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
            (Next::Any, TokenTree::Punct(p)) if p.as_char() == '$' => {
                *next = Next::Ident;
            }
            (Next::Ident, _) => {
                if let TokenTree::Ident(ident) = token {
                    ts.extend(quote! {
                        $($#ident)*
                    });
                    *next = Next::Any;
                    vars.push(ident);
                    continue;
                }
                bail!(token, "expected an identifier");
            }
            _ => {
                ts.extend(token.into_token_stream());
            }
        }
    }
    Ok(ts)
}
