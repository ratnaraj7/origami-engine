use quote::{quote, quote_spanned, ToTokens};
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::{Expr, ExprStruct, Ident, Path, Token, Visibility};

mod children;
mod attributes;
use crate::origami::children::{Children, Context};
use crate::utils::bail;
use crate::utils::kw::{layout, size_hint};

use self::children::{Block, Childrens};

struct Extend {
    // struct of layout to extend
    st: ExprStruct,

    // path of trait of layout to extend
    layout_path: Path,
    blocks: Vec<Block>,
}

enum ChildrenType {
    Normal(Childrens),
    Extended(Extend),
}

struct Layout {
    vis: Visibility,
    name: Ident,
    block_names: Vec<Ident>,
}

pub struct Origami {
    // impl for
    st: Path,

    childrens: ChildrenType,
    size_hint: Option<Expr>,
    layout: Option<Layout>,
}

impl Parse for Origami {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let size_hint = if input.peek(size_hint) {
            input.parse::<size_hint>()?;
            let expr = input.parse::<Expr>()?;
            input.parse::<Token![,]>()?;
            Some(expr)
        } else {
            None
        };

        let layout = if input.peek(layout) {
            input.parse::<layout>()?;
            let vis = input.parse::<Visibility>()?;
            let name = input.parse::<Ident>()?;
            input.parse::<Token![,]>()?;
            Some(Layout {vis, name, block_names: Vec::new()})
        } else {
            None
        };

        let mut children_type = if input.peek(Token![@]) {
            if layout.is_some() {
                bail!(input, "Layouts cannot be extended.");
            }
            input.parse::<Token![@]>()?;
            let st = input.parse::<ExprStruct>()?;
            let layout_path = input.parse::<Path>()?;
            input.parse::<Token![,]>()?;
            ChildrenType::Extended(Extend {
                st,
                layout_path,
                blocks: Vec::new(),
            })
        } else {
            ChildrenType::Normal(Vec::new())
        };

        let st = input.parse::<Path>()?;
        input.parse::<Token![=>]>()?;

        let mut pc = Context {
            is_top_level: true,
            is_extended: matches!(children_type, ChildrenType::Extended(_)),
            block_names: if layout.is_some() {
                Some(Vec::new())
            } else {
                None
            },
            #[cfg(feature = "html_escape")]
            escape: true,
        };

        while !input.is_empty() {
            pc.is_top_level = true;
            #[cfg(feature = "html_escape")]
            if cfg!(feature = "html_escape") {
                pc.escape = true;
            }
            match children_type {
                ChildrenType::Normal(ref mut childrens) => {
                    childrens.push(Children::parse(input, &mut pc)?)
                }
                ChildrenType::Extended(ref mut extend) => match Children::parse(input, &mut pc)? {
                    Children::ExtendBlock(block) => extend.blocks.push(block),
                    _ => unreachable!(),
                },
            }
        }

        Ok(Origami {
            st,
            childrens: children_type,
            size_hint,
            layout: if let Some(mut layout) = layout {
                layout.block_names = pc.block_names.unwrap();
                Some(layout)
            } else {
                None
            },
        })
    }
}

impl ToTokens for Origami {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let st = &self.st;
        let childrens_placeholder = quote! {
            let childrens = |s: &mut String| {};
        };
        let string_init = if let Some(size_hint) = self.size_hint.as_ref() {
            quote! {
                let mut orig = String::with_capacity(#size_hint);
                let s: &mut String = &mut orig;
            }
        } else {
            quote! {
                let mut orig = String::new();
                let s: &mut String = &mut orig;
            }
        };
        let childrens = match &self.childrens {
            ChildrenType::Normal(childrens) => quote! {
                #(#childrens)*
            },
            ChildrenType::Extended(Extend {
                st: ext_st,
                layout_path,
                blocks,
            }) => {
                let span = ext_st.span();
                let mut mthds = Vec::new();
                for Block { name, childrens } in blocks {
                    mthds.push(quote_spanned! {
                        span=>
                            fn #name(&self, s: &mut String) {
                                #(#childrens)*
                            }
                    });
                }
                tokens.extend(quote! {
                    impl #layout_path for #st {
                        #(#mthds)*
                    }
                });
                quote_spanned! {
                    span=>
                    #ext_st.extend(s, self);
                }
            }
        };
        if let Some(Layout { name, block_names, vis}) = self.layout.as_ref() {
            tokens.extend(quote! {
                #vis trait #name {
                    #(
                        fn #block_names(&self, s: &mut String) {}
                    )*
                }

                impl #name for #st {}

                impl<T> ::origami_engine::Layout<T> for #st
                where T: #name {
                    fn extend(&self, s: &mut String, blocks: &T) {
                        #childrens_placeholder
                        #childrens
                    }

                    fn extend_with_childrens(&self, s: &mut String, blocks: &T, childrens: impl Fn(&mut String)) {
                        #childrens
                    }
                }

                impl ::origami_engine::Origami for #st {
                    fn to_html(&self) -> String {
                        #string_init
                        self.extend(s, self);
                        orig
                    }

                    fn push_html_to_string(&self, s: &mut String) {
                        self.extend(s, self);
                    }

                    fn push_html_to_string_with_childrens(&self, s: &mut String, childrens: impl Fn(&mut String)) {
                        self.extend_with_childrens(s, self, childrens);
                    }
                }
            })
        } else {
            tokens.extend(quote! {
                impl ::origami_engine::Origami for #st {
                    fn to_html(&self) -> String {
                        #string_init
                        #childrens_placeholder
                        #childrens
                        orig
                    }

                    fn push_html_to_string(&self, s: &mut String) {
                        #childrens_placeholder
                        #childrens
                    }

                    fn push_html_to_string_with_childrens(&self, s: &mut String, childrens: impl Fn(&mut String)) {
                        #childrens
                    }
                } 
            })
        }


        #[cfg(feature = "axum")]
        tokens.extend(quote! {
            impl ::axum::response::IntoResponse for #st
            {
                fn into_response(self) -> ::axum::response::Response {
                    ::axum::response::Html(self.to_html()).into_response()
                }
            }
        })

    }
}
