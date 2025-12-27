use proc_macro::TokenStream;
use quote::{quote, ToTokens};

use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
};

enum Element {}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        unimplemented!()
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unimplemented!()
    }
}

#[proc_macro]
pub fn soft(input: TokenStream) -> TokenStream {
    let element: Element = parse_macro_input!(input);

    quote! {{
        #element
    }}
    .into()
}
