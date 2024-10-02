use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

use self::anon::Anon;
use self::comp::Component;

mod anon;
mod comp;
mod utils;

#[proc_macro]
pub fn comp(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as Component)
        .into_token_stream()
        .into()
}

#[proc_macro]
pub fn anon(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as Anon).into_token_stream().into()
}
