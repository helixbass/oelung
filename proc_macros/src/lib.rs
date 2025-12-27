use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use squalid::{OptionExtDefault, _d};
use syn::{
    bracketed,
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Ident, LitFloat, LitInt, LitStr, Token,
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
            key => return Err(input.error(format!("Unexpected element `{key}`"))),
        })
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::FlexColumn(flex_column) => quote! { #flex_column },
            Self::Text(text) => quote! { #text },
            Self::Cursor(cursor) => quote! { #cursor },
        }
        .to_tokens(tokens)
    }
}

struct FlexColumn {
    pub children: Vec<Element>,
    pub flex_grow: Option<LitFloat>,
}

impl Parse for FlexColumn {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut children: Option<Vec<Element>> = _d();
        let mut flex_grow: Option<LitFloat> = _d();

        while input.peek(Ident) {
            let key = input.parse::<Ident>().unwrap().to_string();
            match &*key {
                "children" => {
                    assert!(children.is_none(), "Already saw 'children' key");
                    let children_content;
                    bracketed!(children_content in input);
                    let children = children.populate_default();
                    while !children_content.is_empty() {
                        children.push(children_content.parse()?);
                        children_content.parse::<Option<Token![,]>>()?;
                    }
                }
                "flex_grow" => {
                    assert!(flex_grow.is_none(), "Already saw 'flex_grow' key");
                    flex_grow = Some(input.parse()?);
                }
                key => return Err(input.error(format!("Unexpected key `{key}`"))),
            }
        }

        Ok(Self {
            children: children.expect("Expected `children`"),
            flex_grow,
        })
    }
}

impl ToTokens for FlexColumn {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        quote! {
            ::oelung::FlexColumnBuilder::default()
                .child(
                    TextBuilder::default()
                        .text_child("Top area")
                        .cursor_child(Cursor::relative().x(0).y(0))
                        .build()?
                        .into(),
                )
                .flex_grow(1)
                .build()?
                .into()
        }
        .to_tokens(tokens)
    }
}

struct Cursor {
    pub x: LitInt,
    pub y: LitInt,
}

impl Parse for Cursor {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut x: Option<LitInt> = _d();
        let mut y: Option<LitInt> = _d();

        while input.peek(Ident) {
            let key = input.parse::<Ident>().unwrap().to_string();
            match &*key {
                "x" => {
                    assert!(x.is_none(), "Already saw 'x' key");
                    x = Some(input.parse()?);
                }
                "y" => {
                    assert!(y.is_none(), "Already saw 'y' key");
                    y = Some(input.parse()?);
                }
                key => return Err(input.error(format!("Unexpected key `{key}`"))),
            }
        }

        Ok(Self {
            x: x.expect("Expected `x`"),
            y: y.expect("Expected `y`"),
        })
    }
}

struct Text {
    pub text: LitStr,
}

impl Parse for Text {
    fn parse(input: ParseStream) -> Result<Self> {
        let text: LitStr = input.parse()?;

        Ok(Self { text })
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
