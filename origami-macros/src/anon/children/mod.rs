use std::fmt::Debug;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::ParseStream;
use syn::spanned::Spanned;
use syn::token::{Comma, If};
use syn::{braced, Expr, Ident, LitStr, Macro, Pat, Token};

use crate::utils::kw::{escape, noescape};
use crate::utils::{bail, combine_to_lit};

pub(super) mod attributes;
use self::attributes::attribute_to_token;
pub(super) use self::attributes::{AttributeKey, Attributes};

pub(super) type Childrens = Vec<Children>;

pub(super) struct Context {
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
pub(super) enum HtmlChildrens {
    Childrens(Childrens),
    SelfClosing,
}

#[derive(Debug)]
pub(super) struct CustomMatchArm {
    body: Childrens,
    pat: Pat,
    guard: Option<(If, Expr)>,
    comma: Option<Comma>,
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
        #[cfg(feature = "html_escape")]
        escape: bool,
    },
    CompCall {
        comp: Macro,
        #[cfg(feature = "html_escape")]
        escape: bool,
    },
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
    Match {
        expr: Expr,
        arms: Vec<CustomMatchArm>,
    },
    // TODO: pattern matching and handle script tag using macro_rules
    // to make it more convenient in terms of ordering
}

impl Children {
    pub fn parse(input: ParseStream, pc: &mut Context) -> syn::Result<Self> {
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
        if input.peek(Token![@]) {
            return parse_component(input, pc);
        }
        if input.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            let expr: Expr = input.parse()?;
            input.parse::<Token![;]>()?;
            #[cfg(feature = "html_escape")]
            let escape = if input.peek(Token![!]) {
                input.parse::<Token![!]>()?;
                false
            } else {
                pc.escape
            };
            return Ok(Children::Expr {
                expr,
                #[cfg(feature = "html_escape")]
                escape,
            });
        }
        if input.peek(Token![if]) {
            return parse_conditional(input, pc);
        }
        if input.peek(Token![for]) {
            return parse_for(input, pc);
        }
        if input.peek(Token![match]) {
            return parse_match(input, pc);
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
    let comp = input.parse::<Macro>()?;
    input.parse::<Token![;]>()?;
    #[cfg(feature = "html_escape")]
    let escape = if input.peek(Token![!]) {
        input.parse::<Token![!]>()?;
        false
    } else {
        pc.escape
    };
    Ok(Children::CompCall {
        comp,
        #[cfg(feature = "html_escape")]
        escape,
    })
}

fn parse_conditional(input: ParseStream, pc: &mut Context) -> syn::Result<Children> {
    let _span = input.parse::<Token![if]>()?.span();
    let if_cond = input.parse::<Expr>()?;
    input.parse::<Token![;]>()?;
    let if_childrens = parse_block(input, pc)?;
    let mut else_ifs = Vec::new();
    while input.peek(Token![else]) && input.peek2(Token![if]) {
        input.parse::<Token![else]>()?;
        input.parse::<Token![if]>()?;
        let cond = input.parse::<Expr>()?;
        input.parse::<Token![;]>()?;
        let childrens = parse_block(input, pc)?;
        else_ifs.push((cond, childrens));
    }
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
    input.parse::<Token![;]>()?;
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

fn parse_match(input: ParseStream, pc: &mut Context) -> syn::Result<Children> {
    input.parse::<Token![match]>()?;
    let expr: Expr = input.parse()?;
    input.parse::<Token![;]>()?;
    #[cfg(feature = "html_escape")]
    pc.parse_escape_no_escape(input)?;
    let mut arms = Vec::new();
    let content;
    braced!(content in input);
    while !content.is_empty() {
        let pat = Pat::parse_multi(&content)?;
        let guard = if content.peek(Token![if]) {
            Some((content.parse::<Token![if]>()?, content.parse()?))
        } else {
            None
        };
        content.parse::<Token![=>]>()?;
        let body = parse_block(&content, pc)?;
        let comma = if content.peek(Token![,]) {
            Some(content.parse::<Token![,]>()?)
        } else {
            None
        };
        arms.push(CustomMatchArm {
            pat,
            guard,
            body,
            comma,
        })
    }
    Ok(Children::Match { expr, arms })
}

pub(super) struct ChildrensE {
    pub(super) childrens: Childrens,
    pub(super) s: Expr,
}

impl ToTokens for ChildrensE {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let childrens = &self.childrens;
        let s = &self.s;
        for children in childrens {
            tokens.extend(children_to_token(children, s));
        }
    }
}

fn children_to_token(children: &Children, s: &Expr) -> proc_macro2::TokenStream {
    match children {
        Children::Text {
            text,
            #[cfg(feature = "html_escape")]
            escape,
        } => {
            #[cfg(feature = "html_escape")]
            if *escape {
                return quote! {
                    ::origami_engine::encode_text_to_string(#text, &mut #s);
                };
            }
            quote! {
                #s.push_str(#text);
            }
        }
        Children::Expr {
            expr,
            #[cfg(feature = "html_escape")]
            escape,
        } => {
            #[cfg(feature = "html_escape")]
            if *escape {
                return quote! {
                    ::origami_engine::encode_text_to_string(#expr, &mut #s);
                };
            }
            quote! {
                #s.push_str(#expr);
            }
        }
        Children::CompCall {
            comp,
            #[cfg(feature = "html_escape")]
            escape,
        } => {
            let mut comp = comp.clone();
            comp.tokens = {
                #[allow(unused)]
                let mut escape_t = quote! {};
                #[cfg(feature = "html_escape")]
                if *escape {
                    escape_t = quote! {};
                } else {
                    escape_t = quote! {noescape, };
                }
                let ts = comp.tokens;
                quote! {
                    #escape_t
                    #s =>
                    #ts
                }
            };
            quote! {
                #comp;
            }
        }
        Children::Cond {
            if_: (if_expr, if_childrens),
            else_ifs,
            else_,
        } => {
            let mut else_if_tokens = TokenStream::new();
            for (else_if_expr, else_if_childrens) in else_ifs {
                let mut else_if_childrens_t = TokenStream::new();
                for children in else_if_childrens {
                    else_if_childrens_t.extend(children_to_token(children, s));
                }
                else_if_tokens.extend(quote! {
                    else if #else_if_expr {
                        #else_if_childrens_t
                    }
                });
            }
            let mut if_childrens_t = TokenStream::new();
            for children in if_childrens {
                if_childrens_t.extend(children_to_token(children, s));
            }
            let mut else_t = TokenStream::new();
            for children in else_ {
                else_t.extend(children_to_token(children, s));
            }
            quote! {
                if #if_expr {
                    #if_childrens_t
                }
                #else_if_tokens
                else {
                    #else_t
                }
            }
        }
        Children::For {
            expr_b,
            expr_a,
            childrens,
        } => {
            let mut childrens_t = TokenStream::new();
            for children in childrens {
                childrens_t.extend(children_to_token(children, s));
            }
            quote! {
                for #expr_b in #expr_a {
                    #childrens_t
                }
            }
        }
        Children::Html {
            tag,
            attrs,
            childrens,
        } => {
            let tag_span = tag.span();
            let tag = tag.to_string();
            let left_arrow_with_tag = combine_to_lit!(tag_span => "<", tag);
            let close_tag = combine_to_lit!(tag_span => "</", tag, ">");
            let attrs = attribute_to_token(attrs, s);
            match childrens {
                HtmlChildrens::Childrens(childrens) => {
                    let mut childrens_t = TokenStream::new();
                    for children in childrens {
                        childrens_t.extend(children_to_token(children, s));
                    }
                    quote! {
                        #s.push_str(#left_arrow_with_tag);
                        #attrs
                        #s.push('>');
                        #childrens_t
                        #s.push_str(#close_tag);
                    }
                }
                HtmlChildrens::SelfClosing => {
                    quote! {
                        #s.push_str(#left_arrow_with_tag);
                        #attrs
                        #s.push_str("/>");
                    }
                }
            }
        }
        Children::Match { expr, arms } => {
            let mut arms_t = TokenStream::new();
            for CustomMatchArm {
                body,
                pat,
                guard,
                comma,
            } in arms
            {
                let mut childrens_t = TokenStream::new();
                for children in body {
                    childrens_t.extend(children_to_token(children, s));
                }
                let guard = if let Some((if_, expr)) = guard {
                    quote! {
                        #if_ #expr
                    }
                } else {
                    quote! {}
                };
                arms_t.extend(quote! {
                    #pat #guard => {
                        #childrens_t
                    }#comma
                })
            }
            quote! {
                match #expr {
                    #arms_t
                }
            }
        }
    }
}
