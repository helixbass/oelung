use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use squalid::{OptionExtDefault, _d};
use syn::{
    bracketed,
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Expr, Ident, LitFloat, LitInt, LitStr, Token,
};

mod custom_keywords {
    syn::custom_keyword!(FlexColumn);
    syn::custom_keyword!(Text);
    syn::custom_keyword!(Cursor);
}

enum Element {
    FlexColumn(FlexColumn),
    Text(Text),
    Cursor(Cursor),
    Component(Expr),
}

impl Element {
    pub fn into_cursor(self) -> Cursor {
        match self {
            Self::Cursor(cursor) => cursor,
            _ => panic!("expected cursor"),
        }
    }
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![%]>()?;
        Ok(if input.peek(custom_keywords::FlexColumn) {
            let name: Ident = input.parse().unwrap();
            assert_eq!(name.to_string(), "FlexColumn");
            Self::FlexColumn(input.parse()?)
        } else if input.peek(custom_keywords::Text) {
            let name: Ident = input.parse().unwrap();
            assert_eq!(name.to_string(), "Text");
            Self::Text(input.parse()?)
        } else if input.peek(custom_keywords::Cursor) {
            let name: Ident = input.parse().unwrap();
            assert_eq!(name.to_string(), "Cursor");
            Self::Cursor({
                input.parse::<Token![.]>()?;
                let relative: Ident = input.parse()?;
                if relative.to_string() != "Relative" {
                    return Err(input.error(format!("Expected `Relative`")));
                }
                input.parse()?
            })
        } else {
            let component: Expr = input.parse()?;
            Self::Component(component)
        })
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::FlexColumn(flex_column) => {
                quote! { ::oelung::Component::FlexColumn(#flex_column) }
            }
            Self::Text(text) => quote! { ::oelung::Component::Text(#text) },
            Self::Cursor(cursor) => quote! { ::oelung::Component::Cursor(#cursor) },
            Self::Component(component) => {
                quote! { ::oelung::Component::Component(::std::boxed::Box::new(#component)) }
            }
        }
        .to_tokens(tokens)
    }
}

struct FlexColumn {
    pub children: Vec<Element>,
    pub flex_grow: Option<LitFloatOrInt>,
}

impl Parse for FlexColumn {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut children: Option<Vec<Element>> = _d();
        let mut flex_grow: Option<LitFloatOrInt> = _d();

        while input.peek(Ident) {
            let key = input.parse::<Ident>().unwrap().to_string();
            input.parse::<Token![=>]>()?;
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
        let children = self
            .children
            .iter()
            .map(|child| {
                quote! {
                    .child(#child)
                }
            })
            .collect::<Vec<_>>();
        let flex_grow = match self.flex_grow.as_ref() {
            None => quote! {},
            Some(flex_grow) => quote! {
                .flex_grow(#flex_grow)
            },
        };

        quote! {
            ::oelung::FlexColumnBuilder::default()
                #(#children)*
                #flex_grow
                .build()?
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
            input.parse::<Token![=>]>()?;
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

impl ToTokens for Cursor {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let x = &self.x;
        let y = &self.y;

        quote! {
            ::oelung::Cursor::relative().x(#x).y(#y)
        }
        .to_tokens(tokens)
    }
}

struct Text {
    pub text: LitStr,
    pub cursor: Option<Cursor>,
}

impl Parse for Text {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut text: Option<LitStr> = _d();
        let mut cursor: Option<Cursor> = _d();

        match input.peek(LitStr) {
            true => {
                text = input.parse().unwrap();
            }
            false => {
                while input.peek(Ident) {
                    let key = input.parse::<Ident>().unwrap().to_string();
                    input.parse::<Token![=>]>()?;
                    match &*key {
                        "text" => {
                            assert!(text.is_none(), "Already saw 'text' key");
                            text = Some(input.parse()?);
                        }
                        "cursor" => {
                            assert!(cursor.is_none(), "Already saw 'cursor' key");
                            cursor = Some(input.parse::<Element>()?.into_cursor());
                        }
                        key => return Err(input.error(format!("Unexpected key `{key}`"))),
                    }
                }
            }
        }

        Ok(Self {
            text: text.expect("Expected `text`"),
            cursor,
        })
    }
}

impl ToTokens for Text {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let text = &self.text;
        let cursor = match self.cursor.as_ref() {
            None => quote! {},
            Some(cursor) => quote! {
                .cursor_child(#cursor)
            },
        };

        quote! {
            ::oelung::TextBuilder::default()
                .text_child(#text)
                #cursor
                .build()?
        }
        .to_tokens(tokens)
    }
}

enum LitFloatOrInt {
    Float(LitFloat),
    Int(LitInt),
}

impl Parse for LitFloatOrInt {
    fn parse(input: ParseStream) -> Result<Self> {
        match input.peek(LitFloat) {
            true => Ok(Self::Float(input.parse().unwrap())),
            false => Ok(Self::Int(input.parse().unwrap())),
        }
    }
}

impl ToTokens for LitFloatOrInt {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Float(float) => quote! { #float },
            Self::Int(int) => quote! { #int },
        }
        .to_tokens(tokens)
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
