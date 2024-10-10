use std::fmt::Debug;
use std::fs::File;
use std::io::Read;

use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::token::{Comma, If};
use syn::{braced, Expr, Ident, LitStr, Pat, Path, Token};

use crate::utils::kw::{call, i, script, style};
#[cfg(feature = "minify_html")]
use crate::utils::kw::{escape, noescape};
use crate::utils::{bail, combine_to_lit};

pub(super) mod attributes;
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
    pub(super) body: Childrens,
    pub(super) pat: Pat,
    pub(super) guard: Option<(If, Expr)>,
    pub(super) comma: Option<Comma>,
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
        comp: Path,
        ts: TokenStream,
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
    Script {
        text: Option<LitStr>,
        attrs: Attributes,
        #[cfg(feature = "minify_html")]
        minify: bool,
    },
    Style {
        text: Option<LitStr>,
        attrs: Attributes,
        #[cfg(feature = "minify_html")]
        minify: bool,
    },
}

impl Children {
    pub fn parse(input: ParseStream, pc: &mut Context) -> syn::Result<Self> {
        if input.peek(LitStr) || input.peek(i) {
            return parse_text(input, pc);
        }
        if input.peek(style) {
            return parse_style(input);
        }
        if input.peek(script) {
            return parse_script(input);
        }
        if input.peek(call) {
            return parse_component(input, pc);
        }
        if input.peek(Token![@]) {
            input.parse::<Token![@]>()?;
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

#[allow(unused_variables)]
fn parse_text(input: ParseStream, pc: &mut Context) -> syn::Result<Children> {
    let text = if input.peek(i) {
        input.parse::<i>()?;
        let path: LitStr = input.parse()?;
        let mut s = String::new();
        File::open(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path.value()))
            .map_err(|e| syn::Error::new(path.span(), e))?
            .read_to_string(&mut s)
            .map_err(|e| syn::Error::new(path.span(), e))?;
        combine_to_lit!(path.span() => s)
    } else {
        input.parse()?
    };
    Ok(Children::Text {
        text,
        #[cfg(feature = "html_escape")]
        escape: if input.peek(Token![!]) {
            input.parse::<Token![!]>()?;
            false
        } else {
            pc.escape
        },
    })
}

fn parse_component(
    input: ParseStream,
    #[allow(unused_variables)] pc: &mut Context,
) -> syn::Result<Children> {
    input.parse::<call>()?;
    let comp = input.parse()?;
    let content;
    braced!(content in input);
    let ts = content.call(TokenStream::parse)?;
    #[cfg(feature = "html_escape")]
    let escape = if input.peek(Token![!]) {
        input.parse::<Token![!]>()?;
        false
    } else {
        pc.escape
    };
    Ok(Children::CompCall {
        comp,
        ts,
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

fn parse_script(input: ParseStream) -> syn::Result<Children> {
    input.parse::<script>()?;
    let attrs: Attributes = input.parse()?;
    #[cfg(feature = "html_escape")]
    if attrs.0.contains_key(&AttributeKey::Escape) || attrs.0.contains_key(&AttributeKey::NoEscape)
    {
        bail!(
            input,
            "Cannot use `escape` or `noescape` with `script` or `style`"
        );
    }
    #[cfg(feature = "minify_html")]
    let minify = !attrs.0.contains_key(&AttributeKey::NoMinify);
    let mut ctx = Context {
        #[cfg(feature = "html_escape")]
        escape: false,
    };
    let content;
    braced!(content in input);
    let text = if !content.is_empty() {
        match parse_text(&content, &mut ctx)? {
            Children::Text { text, .. } => {
                if !content.is_empty() {
                    bail!(input, "Expected end of `script` block");
                }
                Some(text)
            }
            _ => unreachable!(),
        }
    } else {
        None
    };
    Ok(Children::Script {
        attrs,
        text,
        #[cfg(feature = "minify_html")]
        minify,
    })
}

fn parse_style(input: ParseStream) -> syn::Result<Children> {
    input.parse::<style>()?;
    let attrs: Attributes = input.parse()?;
    #[cfg(feature = "html_escape")]
    if attrs.0.contains_key(&AttributeKey::Escape) || attrs.0.contains_key(&AttributeKey::NoEscape)
    {
        bail!(
            input,
            "Cannot use `escape` or `noescape` with `script` or `style`"
        );
    }
    #[cfg(feature = "minify_html")]
    let minify = !attrs.0.contains_key(&AttributeKey::NoMinify);
    let mut ctx = Context {
        #[cfg(feature = "html_escape")]
        escape: false,
    };
    let content;
    braced!(content in input);
    let text = if !content.is_empty() {
        match parse_text(&content, &mut ctx)? {
            Children::Text { text, .. } => {
                if !content.is_empty() {
                    bail!(input, "Expected end of `style` block");
                }
                Some(text)
            }
            _ => unreachable!(),
        }
    } else {
        None
    };
    Ok(Children::Style {
        attrs,
        text,
        #[cfg(feature = "minify_html")]
        minify,
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
    #[cfg(feature = "minify_html")]
    if attrs.0.contains_key(&AttributeKey::NoMinify) {
        bail!(
            input,
            "`nominify` can only be used with `script` or `style` tags"
        );
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
