#[cfg(feature = "minify_html")]
use minify_html::Cfg;
use proc_macro2::{TokenStream, TokenTree};
use quote::{quote, ToTokens};
use rand::prelude::*;
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::{braced, parse_quote, Expr, Ident, LitStr, Path, Token};

mod children;

use crate::anon::children::CustomMatchArm;
use crate::utils::kw::{childrens, concat_args, concat_args_ident, string};
#[cfg(feature = "html_escape")]
use crate::utils::kw::{escape, noescape};
use crate::utils::{bail, combine_to_lit};

use self::children::attributes::AttributeValue;
use self::children::{AttributeKey, Attributes, Children, Childrens, Context, HtmlChildrens};

pub struct Anon {
    expr: Expr,
    childrens: Childrens,
    concat_args: Option<TokenStream>,
    concat_args_return_ident: Option<Ident>,
}

impl Parse for Anon {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut anon = Self {
            concat_args_return_ident: None,
            concat_args: None,
            childrens: Vec::new(),
            expr: parse_quote!(s),
        };
        let mut count = 0;
        while !input.is_empty() {
            if count > 0 {
                input.parse::<Token![,]>()?;
                if input.is_empty() {
                    break;
                }
            }
            if input.peek(concat_args_ident) {
                if anon.concat_args_return_ident.is_some() {
                    bail!(input, "duplicate `concat_args_ident`");
                }
                input.parse::<concat_args_ident>()?;
                anon.concat_args_return_ident = Some(input.parse()?);
                count += 1;
                continue;
            }
            if input.peek(concat_args) {
                if anon.concat_args.is_some() {
                    bail!(input, "duplicate `concat_args`");
                }
                input.parse::<concat_args>()?;
                let content;
                braced!(content in input);
                anon.concat_args = Some(content.parse()?);
                count += 1;
                continue;
            }
            if input.peek(string) {
                input.parse::<string>()?;
                anon.expr = input.parse()?;
                count += 1;
                continue;
            }
            if input.peek(childrens) {
                if !anon.childrens.is_empty() {
                    bail!(input, "duplicate `childrens`");
                }
                input.parse::<childrens>()?;
                let mut ctx = Context {
                    #[cfg(feature = "html_escape")]
                    escape: if input.peek(escape) {
                        input.parse::<escape>()?;
                        true
                    } else if input.peek(noescape) {
                        input.parse::<noescape>()?;
                        false
                    } else {
                        true
                    },
                };
                let content;
                braced!(content in input);
                while !content.is_empty() {
                    anon.childrens.push(Children::parse(&content, &mut ctx)?);
                }
                count += 1;
                continue;
            }
            bail!(input, "unexpected token");
        }
        Ok(anon)
    }
}

impl ToTokens for Anon {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut concat_args = self.concat_args.clone().unwrap_or_default();
        let mut extend = Extend {
            concat_args: &mut concat_args,
            ts: tokens,
            s: &self.expr,
        };
        extend.extend_childrens(&self.childrens, false);
        if let Some(ident) = &self.concat_args_return_ident {
            tokens.extend(quote! {
                macro_rules! #ident {
                    () => {
                        concat!(#concat_args)
                    }
                }
            });
        } else {
            extend.concat_args_to_concat();
        }
    }
}

#[cfg(feature = "minify_html")]
enum Minify {
    Script,
    Style,
}

enum ProcessType {
    #[cfg(feature = "minify_html")]
    Minify(Minify),
    #[cfg(feature = "html_escape")]
    Escape(bool),
    None,
}

struct Extend<'a> {
    ts: &'a mut TokenStream,
    s: &'a Expr,
    concat_args: &'a mut TokenStream,
}

impl Extend<'_> {
    fn extend<I: IntoIterator<Item = TokenTree>>(&mut self, ts: I) {
        self.ts.extend(ts);
    }

    fn extend_childrens(&mut self, childrens: &[Children], with_brace: bool) {
        if with_brace {
            self.concat_args_to_concat();
            let mut temp_ts = TokenStream::new();
            let mut temp_extend_context = Extend {
                ts: &mut temp_ts,
                s: self.s,
                concat_args: self.concat_args,
            };
            temp_extend_context.extend_childrens(childrens, false);
            temp_extend_context.concat_args_to_concat();
            self.ts.extend(quote! {
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
                    } => self.extend_text(
                        text,
                        #[cfg(feature = "html_escape")]
                        *escape,
                    ),
                    Children::Expr {
                        expr,
                        #[cfg(feature = "html_escape")]
                        escape,
                    } => self.extend_expr(
                        expr,
                        #[cfg(feature = "html_escape")]
                        *escape,
                    ),
                    Children::CompCall {
                        comp,
                        ts: comp_ts,
                        #[cfg(feature = "html_escape")]
                        escape,
                    } => self.extend_comp_call(
                        comp,
                        comp_ts,
                        #[cfg(feature = "html_escape")]
                        *escape,
                    ),
                    Children::Cond {
                        if_,
                        else_ifs,
                        else_,
                    } => self.extend_conditional(if_, else_ifs, else_),
                    Children::For {
                        expr_b,
                        expr_a,
                        childrens,
                    } => self.extend_for(expr_b, expr_a, childrens),
                    Children::Html {
                        tag,
                        attrs,
                        childrens,
                    } => self.extend_html(tag, attrs, childrens),
                    Children::Match { expr, arms } => self.extend_match(expr, arms),
                    Children::Style {
                        text,
                        attrs,
                        #[cfg(feature = "minify_html")]
                        minify,
                    } => self.extend_style(
                        text,
                        attrs,
                        #[cfg(feature = "minify_html")]
                        *minify,
                    ),
                    Children::Script {
                        text,
                        attrs,
                        #[cfg(feature = "minify_html")]
                        minify,
                    } => self.extend_script(
                        text,
                        attrs,
                        #[cfg(feature = "minify_html")]
                        *minify,
                    ),
                }
            }
        }
    }

    fn concat_args_to_concat(&mut self) {
        let s = self.s;
        let concat_args = &mut self.concat_args;
        if !concat_args.is_empty() {
            self.ts.extend(quote! {
                #s.push_str(concat!(#concat_args));
            });
            **concat_args = TokenStream::new();
        }
    }

    fn extend_concat_args(&mut self, literal: &LitStr, pt: ProcessType) {
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
        self.concat_args.extend(quote! {
            #literal,
        })
    }

    fn extend_text(&mut self, text: &LitStr, #[cfg(feature = "html_escape")] escape: bool) {
        #[allow(unused)]
        let mut process_type = ProcessType::None;
        #[allow(clippy::unnecessary_operation)]
        #[cfg(feature = "html_escape")]
        {
            process_type = ProcessType::Escape(escape)
        };
        self.extend_concat_args(text, process_type);
    }

    fn extend_expr(&mut self, expr: &Expr, #[cfg(feature = "html_escape")] escape: bool) {
        self.concat_args_to_concat();
        let s = self.s;
        #[cfg(feature = "html_escape")]
        if escape {
            return self.ts.extend(quote! {
                ::origami_engine::encode_text_to_string(#expr, &mut #s);
            });
        }
        self.ts.extend(quote! {
            #s.push_str(#expr);
        })
    }

    fn extend_comp_call(
        &mut self,
        comp: &Path,
        comp_ts: &TokenStream,
        #[cfg(feature = "html_escape")] escape: bool,
    ) {
        #[allow(unused)]
        let mut escape_ts = quote! {};
        #[cfg(feature = "html_escape")]
        if escape {
            escape_ts = quote! { escape };
        } else {
            escape_ts = quote! { noescape };
        }
        let mut rng = rand::thread_rng();
        let random_number: u64 = rng.gen();
        let concat_args_ident = Ident::new(
            format!(
                "{}_return_{}",
                comp.segments.last().expect("Invalid path").ident,
                random_number
            )
            .as_str(),
            comp.span(),
        );
        let concat_args = &mut self.concat_args;
        let s = self.s;
        self.ts.extend(quote! {
            #comp! {
                @component
                escape { #escape_ts },
                internal {
                    concat_args {
                        #concat_args
                    },
                    concat_args_ident #concat_args_ident,
                    string #s
                },
                #comp_ts
            }
        });
        **concat_args = quote! {
            #concat_args_ident!(),
        };
    }

    fn extend_conditional(
        &mut self,
        (if_expr, if_childrens): &(Expr, Childrens),
        else_ifs: &Vec<(Expr, Childrens)>,
        else_: &Childrens,
    ) {
        self.concat_args_to_concat();
        self.ts.extend(quote! {
            if #if_expr
        });
        self.extend_childrens(if_childrens, true);
        for (else_if_expr, else_if_childrens) in else_ifs {
            self.ts.extend(quote! {
                else if #else_if_expr
            });
            self.extend_childrens(else_if_childrens, true);
        }
        self.ts.extend(quote! {
            else
        });
        self.extend_childrens(else_, true);
    }

    fn extend_for(&mut self, expr_b: &Expr, expr_a: &Expr, childrens: &Childrens) {
        self.concat_args_to_concat();
        self.ts.extend(quote! {
            for #expr_b in #expr_a
        });
        self.extend_childrens(childrens, true);
    }

    fn extend_html(&mut self, tag: &Ident, attrs: &Attributes, childrens: &HtmlChildrens) {
        let tag_span = tag.span();
        let tag = tag.to_string();
        self.extend_concat_args(&combine_to_lit!(tag_span => "<", tag), ProcessType::None);
        self.extend_attributes(attrs);
        match childrens {
            HtmlChildrens::Childrens(childrens) => {
                self.extend_concat_args(&combine_to_lit!(">"), ProcessType::None);
                self.extend_childrens(childrens, false);
                self.extend_concat_args(
                    &combine_to_lit!(tag_span => "</", tag, ">"),
                    ProcessType::None,
                );
            }
            HtmlChildrens::SelfClosing => {
                self.extend_concat_args(&combine_to_lit!("/>"), ProcessType::None);
            }
        }
    }

    fn extend_attributes(&mut self, attributes: &Attributes) {
        for (k, v) in &attributes.0 {
            match k {
                AttributeKey::Ident(ident) => {
                    self.extend_concat_args(
                        &combine_to_lit!(ident.span() => " ", ident.to_string()),
                        ProcessType::None,
                    );
                }
                AttributeKey::LitStr(literal) => {
                    self.extend_concat_args(
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
                        self.extend_concat_args(
                            &combine_to_lit!(literal.span() => "=\"", literal.value(), "\""),
                            ProcessType::None,
                        );
                    }
                    AttributeValue::Expr(expr) => {
                        let s = self.s;
                        self.extend_concat_args(&combine_to_lit!("=\""), ProcessType::None);
                        self.concat_args_to_concat();
                        self.ts.extend(quote! {
                            #s.push_str(#expr);
                        });
                        self.extend_concat_args(&combine_to_lit!("\""), ProcessType::None);
                    }
                }
            }
        }
    }

    fn extend_match(&mut self, expr: &Expr, arms: &[CustomMatchArm]) {
        self.concat_args_to_concat();
        let mut temp = TokenStream::new();
        let mut temp_extend = Extend {
            s: self.s,
            ts: &mut temp,
            concat_args: self.concat_args,
        };
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
            temp_extend.extend(quote! {
                #pat #guard =>
            });
            temp_extend.extend_childrens(body, true);
            temp_extend.concat_args_to_concat();
            temp_extend.extend(quote! {
                #comma
            });
        }
        self.ts.extend(quote! {
            match #expr {
                #temp
            }
        });
    }

    fn extend_style(
        &mut self,
        text: &Option<LitStr>,
        attrs: &Attributes,
        #[cfg(feature = "minify_html")] minify: bool,
    ) {
        self.extend_concat_args(&combine_to_lit!("<style"), ProcessType::None);
        self.extend_attributes(attrs);
        self.extend_concat_args(&combine_to_lit!(">"), ProcessType::None);
        if let Some(text) = text {
            self.extend_concat_args(text, {
                #[allow(unused)]
                let mut process_type = ProcessType::None;
                #[cfg(feature = "minify_html")]
                {
                    if minify {
                        process_type = ProcessType::Minify(Minify::Style);
                    }
                }
                process_type
            });
        }
        self.extend_concat_args(&combine_to_lit!("</style>"), ProcessType::None);
    }

    fn extend_script(
        &mut self,
        text: &Option<LitStr>,
        attrs: &Attributes,
        #[cfg(feature = "minify_html")] minify: bool,
    ) {
        self.extend_concat_args(&combine_to_lit!("<script"), ProcessType::None);
        self.extend_attributes(attrs);
        self.extend_concat_args(&combine_to_lit!(">"), ProcessType::None);
        if let Some(text) = text {
            self.extend_concat_args(text, {
                #[allow(unused)]
                let mut process_type = ProcessType::None;
                #[cfg(feature = "minify_html")]
                {
                    if minify {
                        process_type = ProcessType::Minify(Minify::Script);
                    }
                }
                process_type
            });
        }
        self.extend_concat_args(&combine_to_lit!("</script>"), ProcessType::None);
    }
}
