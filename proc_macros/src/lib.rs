use proc_macro::TokenStream;
use quote::{quote, ToTokens};

use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Ident, Token,
};

enum Element {
    FlexColumn(FlexColumn),
    Text(Text),
    Cursor(Cursor),
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![%]>()?;
        let name: Ident = input.parse()?;

        Ok(match &*name.to_string() {
            "FlexColumn" => Self::FlexColumn(input.parse()?),
            "Text" => Self::Text(input.parse()?),
            "Cursor" => Self::Cursor({
                input.parse::<Token![.]>()?;
                let relative: Ident = input.parse()?;
                if relative.to_string() != "Relative" {
                    return Err(input.error(format!("Expected `Relative`")));
                }
                input.parse()?
            }),
            key => return Err(input.error(format!("Unexpected key `{key}`"))),
        })
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unimplemented!()
    }
}

struct FlexColumn {
    pub children: Vec<Element>,
}

impl Parse for FlexColumn {
    fn parse(input: ParseStream) -> Result<Self> {
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
