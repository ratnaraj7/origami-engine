use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use self::origami::Origami;

mod origami;
mod utils;

#[proc_macro]
pub fn og(input: TokenStream) -> TokenStream {
    let origami = parse_macro_input!(input as Origami);
    quote! {
        #origami
    }
    .into()
}
