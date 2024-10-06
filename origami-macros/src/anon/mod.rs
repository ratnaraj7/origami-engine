#[cfg(feature = "minify_html")]
use minify_html::Cfg;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::parse::Parse;
use syn::token::Return;
use syn::{braced, parse_quote, Expr, Ident, LitStr, Token};

mod children;

use crate::anon::children::CustomMatchArm;
use crate::utils::combine_to_lit;
use crate::utils::kw::literals;
#[cfg(feature = "html_escape")]
use crate::utils::kw::{escape, noescape};

use self::children::attributes::AttributeValue;
use self::children::{
    AttributeKey, Attributes, Children, Childrens, Context, HtmlChildrens, ScriptOrStyleContent,
};

pub struct Anon {
    expr: Expr,
    childrens: Childrens,
    concat_args: Option<TokenStream>,
    concat_args_return_ident: Option<Ident>,
}

impl Parse for Anon {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let concat_args = if input.peek(literals) {
            input.parse::<literals>()?;
            let content;
            braced!(content in input);
            input.parse::<Token![,]>()?;
            Some(content.parse()?)
        } else {
            None
        };
        let concat_args_return_ident = if input.peek(Return) {
            input.parse::<Return>()?;
            let ident = Some(input.parse()?);
            input.parse::<Token![,]>()?;
            ident
        } else {
            None
        };
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
            expr,
            childrens,
            concat_args,
            concat_args_return_ident,
        })
    }
}

impl ToTokens for Anon {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut concat_args = self.concat_args.clone().unwrap_or_default();
        childrens_to_tokens(tokens, &self.childrens, &self.expr, &mut concat_args, false);
        if let Some(ident) = &self.concat_args_return_ident {
            tokens.extend(quote! {
                macro_rules! #ident {
                    () => {
                        concat!(#concat_args)
                    }
                }
            });
        } else {
            concat_args_to_concat(tokens, &mut concat_args, &self.expr);
        }
    }
}

#[cfg(feature = "minify_html")]
enum Minify {
    Script,
    Style,
    None,
}

enum ProcessType {
    #[cfg(feature = "minify_html")]
    Minify(Minify),
    #[cfg(feature = "html_escape")]
    Escape(bool),
    None,
}

fn extend_concat_args(concat_args: &mut TokenStream, literal: &LitStr, pt: ProcessType) {
    let literal = match pt {
        #[cfg(feature = "minify_html")]
        ProcessType::Minify(Minify::Script) => {
            let cfg = Cfg {
                minify_js: true,
                ..Default::default()
            };
            let value = minify_html::minify(literal.value().as_bytes(), &cfg);
            let value = String::from_utf8_lossy(value.as_slice());
            &LitStr::new(&value, literal.span())
        }
        #[cfg(feature = "minify_html")]
        ProcessType::Minify(Minify::Style) => {
            let cfg = Cfg {
                minify_css: true,
                ..Default::default()
            };
            let value = minify_html::minify(literal.value().as_bytes(), &cfg);
            let value = String::from_utf8_lossy(value.as_slice());
            &LitStr::new(&value, literal.span())
        }
        #[cfg(feature = "html_escape")]
        ProcessType::Escape(escape) if escape => {
            let value = literal.value();
            let value = html_escape::encode_text(value.as_str());
            &LitStr::new(value.as_ref(), literal.span())
        }
        _ => literal,
    };
    concat_args.extend(quote! {
        #literal,
    })
}

fn concat_args_to_concat(ts: &mut TokenStream, concat_args: &mut TokenStream, s: &Expr) {
    if !concat_args.is_empty() {
        ts.extend(quote! {
            #s.push_str(concat!(#concat_args));
        });
        *concat_args = TokenStream::new();
    }
}

fn extend_text(
    text: &LitStr,
    concat_args: &mut TokenStream,
    #[cfg(feature = "html_escape")] escape: bool,
) {
    #[allow(unused)]
    let mut process_type = ProcessType::None;
    #[allow(clippy::unnecessary_operation)]
    #[cfg(feature = "html_escape")]
    {
        process_type = ProcessType::Escape(escape)
    };
    extend_concat_args(concat_args, text, process_type);
}

fn expr_to_tokens(
    ts: &mut TokenStream,
    expr: &Expr,
    s: &Expr,
    concat_args: &mut TokenStream,
    #[cfg(feature = "html_escape")] escape: bool,
) {
    concat_args_to_concat(ts, concat_args, s);
    #[cfg(feature = "html_escape")]
    if escape {
        return ts.extend(quote! {
            ::origami_engine::encode_text_to_string(#expr, &mut #s);
        });
    }
    ts.extend(quote! {
        #s.push_str(#expr);
    })
}

fn conditonal_to_tokens(
    ts: &mut TokenStream,
    (if_expr, if_childrens): &(Expr, Childrens),
    else_ifs: &Vec<(Expr, Childrens)>,
    else_: &Childrens,
    s: &Expr,
    concat_args: &mut TokenStream,
) {
    concat_args_to_concat(ts, concat_args, s);
    ts.extend(quote! {
        if #if_expr
    });
    childrens_to_tokens(ts, if_childrens, s, concat_args, true);
    for (else_if_expr, else_if_childrens) in else_ifs {
        ts.extend(quote! {
            else if #else_if_expr
        });
        childrens_to_tokens(ts, else_if_childrens, s, concat_args, true);
    }
    ts.extend(quote! {
        else
    });
    childrens_to_tokens(ts, else_, s, concat_args, true);
}

fn comp_call_to_tokens(
    ts: &mut TokenStream,
    comp: &Ident,
    comp_ts: &TokenStream,
    s: &Expr,
    concat_args: &mut TokenStream,
    #[cfg(feature = "html_escape")] escape: bool,
) {
    #[allow(unused)]
    let mut escape_ts = quote! {};
    #[cfg(feature = "html_escape")]
    if escape {
        escape_ts = quote! {};
    } else {
        escape_ts = quote! {noescape, };
    }
    let return_ident = Ident::new(format!("{}_return", comp,).as_str(), comp.span());
    ts.extend(quote! {
        #comp! {
            literals {
                #concat_args
            },
            return #return_ident,
            #escape_ts
            #s =>
            #comp_ts
        }
    });
    *concat_args = quote! {
        #return_ident!(),
    };
}

fn loop_to_tokens(
    ts: &mut TokenStream,
    expr_b: &Expr,
    expr_a: &Expr,
    childrens: &Childrens,
    s: &Expr,
    concat_args: &mut TokenStream,
) {
    concat_args_to_concat(ts, concat_args, s);
    ts.extend(quote! {
        for #expr_b in #expr_a
    });
    childrens_to_tokens(ts, childrens, s, concat_args, true);
}

fn html_to_tokens(
    ts: &mut TokenStream,
    tag: &Ident,
    attrs: &Attributes,
    childrens: &HtmlChildrens,
    s: &Expr,
    concat_args: &mut TokenStream,
) {
    let tag_span = tag.span();
    let tag = tag.to_string();
    extend_concat_args(
        concat_args,
        &combine_to_lit!(tag_span => "<", tag),
        ProcessType::None,
    );
    attribute_to_token(ts, attrs, s, concat_args);
    match childrens {
        HtmlChildrens::Childrens(childrens) => {
            extend_concat_args(concat_args, &combine_to_lit!(">"), ProcessType::None);
            childrens_to_tokens(ts, childrens, s, concat_args, false);
            extend_concat_args(
                concat_args,
                &combine_to_lit!(tag_span => "</", tag, ">"),
                ProcessType::None,
            );
        }
        HtmlChildrens::SelfClosing => {
            extend_concat_args(concat_args, &combine_to_lit!("/>"), ProcessType::None);
        }
    }
}

fn match_to_tokens(
    ts: &mut TokenStream,
    expr: &Expr,
    arms: &[CustomMatchArm],
    s: &Expr,
    concat_args: &mut TokenStream,
) {
    concat_args_to_concat(ts, concat_args, s);
    let mut temp = TokenStream::new();
    for CustomMatchArm {
        body,
        pat,
        guard,
        comma,
    } in arms
    {
        let guard = if let Some((if_, expr)) = guard {
            quote! {
                #if_ #expr
            }
        } else {
            quote! {}
        };
        temp.extend(quote! {
            #pat #guard =>
        });
        childrens_to_tokens(&mut temp, body, s, concat_args, true);
        concat_args_to_concat(&mut temp, concat_args, s);
        temp.extend(quote! {
            #comma
        });
    }
    ts.extend(quote! {
        match #expr {
            #temp
        }
    });
}

fn style_to_tokens(
    ts: &mut TokenStream,
    ty: &ScriptOrStyleContent,
    attrs: &Attributes,
    concat_args: &mut TokenStream,
    s: &Expr,
    #[cfg(feature = "minify_html")] minify: bool,
) {
    extend_concat_args(concat_args, &combine_to_lit!("<style"), ProcessType::None);
    attribute_to_token(ts, attrs, s, concat_args);
    extend_concat_args(concat_args, &combine_to_lit!(">"), ProcessType::None);
    script_or_style_content_to_tokens(
        ts,
        ty,
        concat_args,
        s,
        #[cfg(feature = "minify_html")]
        if minify { Minify::Style } else { Minify::None },
    );
    extend_concat_args(concat_args, &combine_to_lit!("</style>"), ProcessType::None);
}

fn script_to_tokens(
    ts: &mut TokenStream,
    name: &Option<LitStr>,
    ty: &ScriptOrStyleContent,
    attrs: &Attributes,
    concat_args: &mut TokenStream,
    s: &Expr,
    #[cfg(feature = "minify_html")] minify: bool,
) {
    if let Some(name) = name {
        let mut temp = TokenStream::new();
        let mut temp_concat_args = TokenStream::new();
        extend_concat_args(
            &mut temp_concat_args,
            &combine_to_lit!("<script"),
            ProcessType::None,
        );
        attribute_to_token(&mut temp, attrs, s, concat_args);
        extend_concat_args(
            &mut temp_concat_args,
            &combine_to_lit!(">"),
            ProcessType::None,
        );
        script_or_style_content_to_tokens(
            &mut temp,
            ty,
            &mut temp_concat_args,
            s,
            #[cfg(feature = "minify_html")]
            if minify { Minify::Script } else { Minify::None },
        );
        extend_concat_args(
            &mut temp_concat_args,
            &combine_to_lit!("</script>"),
            ProcessType::None,
        );
        concat_args_to_concat(&mut temp, &mut temp_concat_args, s);
        let name = Ident::new(name.value().as_str(), name.span());
        ts.extend(quote! {
            macro_rules! #name {
                (javascript) => {
                    #temp
                }
            }
        });
    } else {
        extend_concat_args(concat_args, &combine_to_lit!("<script"), ProcessType::None);
        attribute_to_token(ts, attrs, s, concat_args);
        extend_concat_args(concat_args, &combine_to_lit!(">"), ProcessType::None);
        script_or_style_content_to_tokens(
            ts,
            ty,
            concat_args,
            s,
            #[cfg(feature = "minify_html")]
            if minify { Minify::Script } else { Minify::None },
        );
        extend_concat_args(
            concat_args,
            &combine_to_lit!("</script>"),
            ProcessType::None,
        );
    }
}

fn script_or_style_content_to_tokens(
    ts: &mut TokenStream,
    ty: &ScriptOrStyleContent,
    concat_args: &mut TokenStream,
    s: &Expr,
    #[cfg(feature = "minify_html")] minify: Minify,
) {
    #[allow(unused)]
    let mut process_type = ProcessType::None;
    #[cfg(feature = "minify_html")]
    {
        process_type = ProcessType::Minify(minify);
    }
    match ty {
        ScriptOrStyleContent::LitStr(lit) => {
            extend_concat_args(concat_args, lit, process_type);
        }
        ScriptOrStyleContent::Expr(expr) => {
            concat_args_to_concat(ts, concat_args, s);
            match process_type {
                #[cfg(feature = "minify_html")]
                ProcessType::Minify(Minify::Script) => {
                    ts.extend(quote! {{
                        let temp_s = String::from_utf8_lossy(::origami_engine::minify(#expr.as_bytes(), &::origami_engine::Cfg{minify_js: true, ..Default::default()}).as_slice()).to_string();
                        #s.push_str(temp_s.as_str());
                    }});
                }
                #[cfg(feature = "minify_html")]
                ProcessType::Minify(Minify::Style) => {
                    ts.extend(quote! {{
                        let temp_s = String::from_utf8_lossy(::origami_engine::minify(#expr.as_bytes(), &::origami_engine::Cfg{minify_css: true, ..Default::default()}).as_slice()).to_string();
                        #s.push_str(temp_s.as_str());
                    }});
                }
                _ => {
                    ts.extend(quote! {
                        #s.push_str(#expr);
                    });
                }
            }
        }
        ScriptOrStyleContent::Empty => {}
    }
}

fn attribute_to_token(
    ts: &mut TokenStream,
    attributes: &Attributes,
    s: &Expr,
    concat_args: &mut TokenStream,
) {
    for (k, v) in &attributes.0 {
        match k {
            AttributeKey::Ident(ident) if ident == "script_name" => {
                continue;
            }
            AttributeKey::Ident(ident) => {
                extend_concat_args(
                    concat_args,
                    &combine_to_lit!(ident.span() => " ", ident.to_string()),
                    ProcessType::None,
                );
            }
            AttributeKey::LitStr(literal) => {
                extend_concat_args(
                    concat_args,
                    &combine_to_lit!(literal.span() => " ", literal.value()),
                    ProcessType::None,
                );
            }
            #[cfg(feature = "html_escape")]
            AttributeKey::Escape | AttributeKey::NoEscape => {
                continue;
            }
            #[cfg(feature = "minify_html")]
            AttributeKey::NoMinify => {
                continue;
            }
        }
        if let Some(v) = v {
            match v {
                AttributeValue::LitStr(literal) => {
                    extend_concat_args(
                        concat_args,
                        &combine_to_lit!(literal.span() => "=\"", literal.value(), "\""),
                        ProcessType::None,
                    );
                }
                AttributeValue::Expr(expr) => {
                    extend_concat_args(concat_args, &combine_to_lit!("=\""), ProcessType::None);
                    concat_args_to_concat(ts, concat_args, s);
                    ts.extend(quote! {
                        #s.push_str(#expr);
                    });
                    extend_concat_args(concat_args, &combine_to_lit!("\""), ProcessType::None);
                }
            }
        }
    }
}

fn childrens_to_tokens(
    ts: &mut TokenStream,
    childrens: &[Children],
    s: &Expr,
    concat_args: &mut TokenStream,
    with_brace: bool,
) {
    if with_brace {
        let mut temp_ts = TokenStream::new();
        childrens_to_tokens(&mut temp_ts, childrens, s, concat_args, false);
        concat_args_to_concat(&mut temp_ts, concat_args, s);
        ts.extend(quote! {
            {
                #temp_ts
            }
        })
    } else {
        for children in childrens {
            match children {
                Children::Text {
                    text,
                    #[cfg(feature = "html_escape")]
                    escape,
                } => extend_text(
                    text,
                    concat_args,
                    #[cfg(feature = "html_escape")]
                    *escape,
                ),
                Children::Expr {
                    expr,
                    #[cfg(feature = "html_escape")]
                    escape,
                } => expr_to_tokens(
                    ts,
                    expr,
                    s,
                    concat_args,
                    #[cfg(feature = "html_escape")]
                    *escape,
                ),
                Children::CompCall {
                    comp,
                    ts: comp_ts,
                    #[cfg(feature = "html_escape")]
                    escape,
                } => comp_call_to_tokens(
                    ts,
                    comp,
                    comp_ts,
                    s,
                    concat_args,
                    #[cfg(feature = "html_escape")]
                    *escape,
                ),
                Children::Cond {
                    if_,
                    else_ifs,
                    else_,
                } => conditonal_to_tokens(ts, if_, else_ifs, else_, s, concat_args),
                Children::For {
                    expr_b,
                    expr_a,
                    childrens,
                } => loop_to_tokens(ts, expr_b, expr_a, childrens, s, concat_args),
                Children::Html {
                    tag,
                    attrs,
                    childrens,
                } => html_to_tokens(ts, tag, attrs, childrens, s, concat_args),
                Children::Match { expr, arms } => match_to_tokens(ts, expr, arms, s, concat_args),
                Children::Style {
                    ty,
                    attrs,
                    #[cfg(feature = "minify_html")]
                    minify,
                } => style_to_tokens(
                    ts,
                    ty,
                    attrs,
                    concat_args,
                    s,
                    #[cfg(feature = "minify_html")]
                    *minify,
                ),
                Children::Script {
                    name,
                    ty,
                    attrs,
                    #[cfg(feature = "minify_html")]
                    minify,
                } => script_to_tokens(
                    ts,
                    name,
                    ty,
                    attrs,
                    concat_args,
                    s,
                    #[cfg(feature = "minify_html")]
                    *minify,
                ),
                Children::ScriptUse { ident } => {
                    concat_args_to_concat(ts, concat_args, s);
                    ts.extend(quote_spanned! {
                        ident.span() =>
                        #ident!(javascript);
                    })
                }
            }
        }
    }
}
