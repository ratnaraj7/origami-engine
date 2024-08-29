use std::fmt::Debug;

use proc_macro2::Span;
use quote::{quote, quote_spanned, ToTokens};
use syn::parse::ParseStream;
use syn::spanned::Spanned;
use syn::token::{Brace, Paren};
use syn::{braced, parenthesized, Expr, ExprStruct, Ident, LitChar, LitStr, Token};

use crate::utils::bail;
use crate::utils::kw::{block, childrens, escape, extend, include, noescape};

use super::attributes::{AttributeKey, Attributes};

pub(super) type Childrens = Vec<Children>;

#[derive(Debug)]
pub(super) struct Block {
    pub(super) name: Ident,
    pub(super) childrens: Childrens,
}

pub(super) struct Context {
    pub(super) is_top_level: bool,
    pub(super) is_extended: bool,
    pub(super) block_names: Option<Vec<Ident>>,
    #[cfg(feature = "html_escape")]
    pub(super) escape: bool,
}

impl Context {
    #[cfg(feature = "html_escape")]
    fn parse_escape_no_escape(&mut self, input: ParseStream) -> syn::Result<()> {
        if input.peek(escape) {
            input.parse::<escape>()?;
            self.escape = true;
        } else if input.peek(noescape) {
            input.parse::<noescape>()?;
            self.escape = false;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(super) enum Component {
    Normal {
        st_expr: ExprStruct,
        childrens: Childrens,
    },
    IteratingExpr(Expr),
    Expr(Expr),
}

#[derive(Debug)]
pub(super) enum HtmlChildrens {
    Childrens(Childrens),
    SelfClosing,
}

#[derive(Debug)]
pub(super) enum Children {
    Text {
        text: LitStr,
        #[cfg(feature = "html_escape")]
        escape: bool,
    },
    Expr {
        expr: Expr,
        iterating: bool,
        #[cfg(feature = "html_escape")]
        escape: bool,
    },
    Comp(Component),
    Cond {
        if_: (Expr, Childrens),
        else_ifs: Vec<(Expr, Childrens)>,
        else_: Childrens,
    },
    For {
        expr_b: Expr,
        expr_a: Expr,
        childrens: Childrens,
    },
    Html {
        tag: Ident,
        attrs: Attributes,
        childrens: HtmlChildrens,
    },
    Include {
        expr: Expr,
        #[cfg(feature = "html_escape")]
        escape: bool,
    },
    ComponentChildrenUse(Span),
    ExtendBlock(Block),
    LayoutBlockUse(Ident),
}

impl Children {
    pub fn parse(input: ParseStream, pc: &mut Context) -> syn::Result<Self> {
        // if extend blocks is provided it means it is extended
        if pc.is_extended && pc.is_top_level && !input.peek(extend) {
            bail!(input, "Components that extend layout can only contain blocks with 'extend_block' keyword at top level.")
        }

        if input.peek(extend) {
            return parse_extend_block(input, pc);
        }

        if input.peek(LitStr) {
            let text: LitStr = input.parse()?;
            return Ok(Children::Text {
                text,
                #[cfg(feature = "html_escape")]
                escape: if input.peek(Token![!]) {
                    input.parse::<Token![!]>()?;
                    false
                } else {
                    pc.escape
                },
            });
        }

        if input.peek(Paren) {
            let content;
            parenthesized!(content in input);
            let iterating = if content.peek(Token![..]) {
                content.parse::<Token![..]>()?;
                true
            } else {
                false
            };
            let expr: Expr = content.parse()?;
            #[cfg(feature = "html_escape")]
            let escape = if input.peek(Token![!]) {
                input.parse::<Token![!]>()?;
                false
            } else {
                pc.escape
            };
            input.parse::<Token![;]>()?;
            return Ok(Children::Expr {
                expr,
                iterating,
                #[cfg(feature = "html_escape")]
                escape,
            });
        }

        if input.peek(Token![@]) {
            return parse_component(input, pc);
        }

        if input.peek(Token![if]) {
            return parse_conditional(input, pc);
        }

        if input.peek(include) {
            input.parse::<include>()?;
            let expr = input.parse::<Expr>()?;
            #[cfg(feature = "html_escape")]
            let escape = if input.peek(Token![!]) {
                input.parse::<Token![!]>()?;
                false
            } else {
                pc.escape
            };
            input.parse::<Token![;]>()?;
            return Ok(Children::Include {
                expr,
                #[cfg(feature = "html_escape")]
                escape,
            });
        }

        if input.peek(childrens) {
            let span = input.parse::<childrens>()?.span();
            input.parse::<Token![;]>()?;
            return Ok(Children::ComponentChildrenUse(span));
        }

        if input.peek(block) {
            if pc.block_names.is_none() {
                bail!(input, "Blocks can only be used in layout.")
            }
            input.parse::<block>()?;
            let ident = input.parse::<Ident>()?;
            input.parse::<Token![;]>()?;
            pc.block_names.as_mut().unwrap().push(ident.clone());
            return Ok(Children::LayoutBlockUse(ident));
        }

        if input.peek(Token![for]) {
            return parse_for(input, pc);
        }

        if input.peek(Ident) {
            return parse_html(input, pc);
        }

        bail!(
            input,
            "Invalid input for `Children`. Expected one of the following: `LitStr`, `if`, `@`, `self`/`..`, or `Ident`."
        )
    }
}

fn parse_block(input: ParseStream, pc: &mut Context) -> syn::Result<Childrens> {
    // always false because childrens inside block can't be top level
    pc.is_top_level = false;
    #[cfg(feature = "html_escape")]
    pc.parse_escape_no_escape(input)?;
    let content;
    braced!(content in input);
    let mut childrens = Vec::new();
    while !content.is_empty() {
        childrens.push(Children::parse(&content, pc)?);
    }
    Ok(childrens)
}

fn parse_component(input: ParseStream, pc: &mut Context) -> syn::Result<Children> {
    input.parse::<Token![@]>()?;
    if input.peek(Paren) {
        let content;
        parenthesized!(content in input);
        if content.peek(Token![..]) {
            content.parse::<Token![..]>()?;
            let expr = content.parse()?;
            input.parse::<Token![;]>()?;
            return Ok(Children::Comp(Component::IteratingExpr(expr)));
        } else {
            let expr = content.parse()?;
            input.parse::<Token![;]>()?;
            return Ok(Children::Comp(Component::Expr(expr)));
        }
    }
    let st_expr: ExprStruct = input.parse()?;
    if input.peek2(Brace) || input.peek(Brace) {
        return Ok(Children::Comp(Component::Normal {
            st_expr,
            childrens: parse_block(input, pc)?,
        }));
    }
    input.parse::<Token![;]>()?;
    Ok(Children::Comp(Component::Normal {
        st_expr,
        childrens: Vec::new(),
    }))
}

fn parse_conditional(input: ParseStream, pc: &mut Context) -> syn::Result<Children> {
    let _span = input.parse::<Token![if]>()?.span();

    // Parse the `if` condition and block
    let if_cond = input.parse::<Expr>()?;
    let if_childrens = parse_block(input, pc)?;

    // Parse `else if` blocks
    let mut else_ifs = Vec::new();
    while input.peek(Token![else]) && input.peek2(Token![if]) {
        input.parse::<Token![else]>()?;
        input.parse::<Token![if]>()?;
        let cond = input.parse::<Expr>()?;
        let childrens = parse_block(input, pc)?;
        else_ifs.push((cond, childrens));
    }

    // Parse the `else` block
    let else_ = if input.peek(Token![else]) {
        input.parse::<Token![else]>()?;
        parse_block(input, pc)?
    } else {
        Vec::new()
    };

    Ok(Children::Cond {
        if_: (if_cond, if_childrens),
        else_ifs,
        else_,
    })
}

fn parse_for(input: ParseStream, pc: &mut Context) -> syn::Result<Children> {
    input.parse::<Token![for]>()?;
    let expr_b: Expr = input.parse()?;
    input.parse::<Token![in]>()?;
    let expr_a: Expr = input.parse()?;
    let childrens = parse_block(input, pc)?;
    Ok(Children::For {
        expr_b,
        expr_a,
        childrens,
    })
}

fn parse_html(input: ParseStream, pc: &mut Context) -> syn::Result<Children> {
    let tag: Ident = input.parse()?;

    let attrs: Attributes = input.parse()?;
    #[cfg(feature = "html_escape")]
    if attrs.0.contains_key(&AttributeKey::Escape) {
        pc.escape = true;
    }
    #[cfg(feature = "html_escape")]
    if attrs.0.contains_key(&AttributeKey::NoEscape) {
        pc.escape = false;
    }

    if input.peek(Token![;]) {
        input.parse::<Token![;]>()?;
        return Ok(Children::Html {
            tag,
            attrs,
            childrens: HtmlChildrens::SelfClosing,
        });
    }

    Ok(Children::Html {
        tag,
        attrs,
        childrens: HtmlChildrens::Childrens(parse_block(input, pc)?),
    })
}

fn parse_extend_block(input: ParseStream, pc: &mut Context) -> syn::Result<Children> {
    if !pc.is_top_level {
        bail!(
            input,
            "Extended blocks must remain at the top level and cannot be nested."
        )
    }
    if !pc.is_extended {
        bail!(
            input,
            "Components that do not extend cannot contain extended block."
        )
    }
    input.parse::<extend>()?;
    let name: Ident = input.parse()?;
    let childrens = parse_block(input, pc)?;
    Ok(Children::ExtendBlock(Block { name, childrens }))
}

impl ToTokens for Children {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Children::Text {
                text,
                #[cfg(feature = "html_escape")]
                escape,
            } => {
                #[cfg(feature = "html_escape")]
                if *escape {
                    return tokens.extend(quote! {
                        ::origami_engine::encode_text_to_string(#text, s);
                    });
                }
                tokens.extend(quote! {
                    s.push_str(#text);
                })
            }
            Children::Expr {
                expr,
                iterating,
                #[cfg(feature = "html_escape")]
                escape,
            } => {
                if *iterating {
                    let span = expr.span();
                    #[cfg(feature = "html_escape")]
                    if *escape {
                        return tokens.extend(quote_spanned! {
                            span=>
                            ::origami_engine::encode_text_to_string(#expr.join(""), s);
                        });
                    }
                    tokens.extend(quote_spanned! {
                        span=>
                        s.push_str(#expr.join("").as_str());
                    })
                } else {
                    let span = expr.span();
                    #[cfg(feature = "html_escape")]
                    if *escape {
                        return tokens.extend(quote_spanned! {
                            span=>
                            ::origami_engine::encode_text_to_string(#expr,s);
                        });
                    }
                    tokens.extend(quote_spanned! {
                        span=>
                        s.push_str(#expr);
                    })
                }
            }
            Children::Comp(comp) => match comp {
                Component::Normal { st_expr, childrens } => {
                    let span = st_expr.span();
                    tokens.extend(quote_spanned! {
                        span=>
                        #[allow(unused_variables)]
                        #st_expr.push_html_to_string_with_childrens(s, |s|{
                            #(#childrens)*
                        });
                    })
                }
                Component::Expr(expr) => {
                    let span = expr.span();
                    tokens.extend(quote_spanned! {
                        span =>
                        #expr.push_html_to_string(s);
                    })
                }
                Component::IteratingExpr(expr) => {
                    let span = expr.span();
                    tokens.extend(quote_spanned! {
                        span=>
                        for comp in #expr {
                            comp.push_html_to_string(s);
                        }
                    })
                }
            },
            Children::Cond {
                if_: (if_expr, if_childrens),
                else_ifs,
                else_,
            } => {
                let span = if_expr.span();
                tokens.extend(quote_spanned! {
                    span=>
                    if #if_expr {
                        #(#if_childrens)*
                    }
                });
                for (else_if_expr, else_if_childrens) in else_ifs {
                    let span = else_if_expr.span();
                    tokens.extend(quote_spanned! {
                        span=>
                        else if #else_if_expr {
                            #(#else_if_childrens)*
                        }
                    });
                }
                tokens.extend(quote! {
                    else {
                        #(#else_)*
                    }
                });
            }
            Children::For {
                expr_b,
                expr_a,
                childrens,
            } => {
                tokens.extend(quote! {
                    for #expr_b in #expr_a {
                        #(#childrens)*
                    }
                });
            }
            Children::ExtendBlock(_) => {
                unreachable!()
            }
            Children::Html {
                tag,
                attrs,
                childrens,
            } => {
                let tag_span = tag.span();
                let tag = tag.to_string();
                let tlen = tag.len();
                let tag = if tlen == 1 {
                    let tag_ch = LitChar::new(tag.chars().next().unwrap(), tag_span);
                    quote_spanned! {
                        tag_span=>
                        s.push(#tag_ch);
                    }
                } else {
                    let tag_str = LitStr::new(&tag, tag_span);
                    quote_spanned! {
                        tag_span=>
                        s.push_str(#tag_str);
                    }
                };

                match childrens {
                    HtmlChildrens::Childrens(childrens) => {
                        tokens.extend(quote_spanned! {
                            tag_span=>
                            s.push('<');
                            #tag
                            #attrs
                            s.push('>');
                            #(#childrens)*
                            s.push_str("</");
                            #tag
                            s.push('>');
                        });
                    }
                    HtmlChildrens::SelfClosing => {
                        tokens.extend(quote_spanned! {
                            tag_span=>
                            s.push('<');
                            #tag
                            #attrs
                            s.push_str("/>");
                        });
                    }
                }
            }
            Children::Include {
                expr,
                #[cfg(feature = "html_escape")]
                escape,
            } => {
                let span = expr.span();
                #[cfg(feature = "html_escape")]
                if *escape {
                    return tokens.extend(quote_spanned! {
                        span=>
                        ::origami_engine::encode_text_to_string(include_str!(#expr), s);
                    });
                }
                tokens.extend(quote_spanned! {
                    span=>
                    s.push_str(include_str!(#expr));
                })
            }
            Children::ComponentChildrenUse(span) => {
                let span = *span;
                tokens.extend(quote_spanned! {
                    span=>
                    childrens(s);
                })
            }
            Children::LayoutBlockUse(ident) => {
                let span = ident.span();
                tokens.extend(quote_spanned! {
                    span=>
                    blocks.#ident(s);
                })
            }
        }
    }
}
