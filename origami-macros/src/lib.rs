use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use self::anon::Anon;
use self::comp::Component;

mod anon;
mod comp;
mod utils;

#[proc_macro]
pub fn comp(input: TokenStream) -> TokenStream {
    let comp = parse_macro_input!(input as Component);
    quote! {
        #comp
    }
    .into()
}

#[proc_macro]
pub fn anon(input: TokenStream) -> TokenStream {
    let anon = parse_macro_input!(input as Anon);
    quote! {
        #anon
    }
    .into()
}
