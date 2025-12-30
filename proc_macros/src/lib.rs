use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use squalid::{OptionExtDefault, _d};
use syn::{
    bracketed, parenthesized,
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Expr, Ident, LessThanBinaryExpr, LitFloat, LitInt, LitStr, Token,
};

mod custom_keywords {
    syn::custom_keyword!(FlexColumn);
    syn::custom_keyword!(Text);
    syn::custom_keyword!(children);
}

enum Element {
    FlexColumn(FlexColumn),
    Text(Text),
    Component(Expr),
    AlreadyComponent(Expr),
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        let me_percent_sign_start_column = input.span().start().column;
        let me_percent_sign_row = input.span().start().line;
        Ok(match input.peek(Token![%]) {
            true => {
                input.parse::<Token![%]>().unwrap();
                if input.peek(custom_keywords::FlexColumn) {
                    let name: Ident = input.parse().unwrap();
                    assert_eq!(name.to_string(), "FlexColumn");
                    Self::FlexColumn(
                        illicit::Layer::new()
                            .offer(MePercentSignStartColumn(me_percent_sign_start_column))
                            .offer(MePercentSignRow(me_percent_sign_row))
                            .enter(|| -> Result<FlexColumn> { Ok(input.parse()?) })?,
                    )
                } else if input.peek(custom_keywords::Text) {
                    let name: Ident = input.parse().unwrap();
                    assert_eq!(name.to_string(), "Text");
                    Self::Text(
                        illicit::Layer::new()
                            .offer(MePercentSignStartColumn(me_percent_sign_start_column))
                            .offer(MePercentSignRow(me_percent_sign_row))
                            .enter(|| -> Result<Text> { Ok(input.parse()?) })?,
                    )
                } else {
                    let component = input.parse::<LessThanBinaryExpr>()?.expr;
                    Self::Component(component)
                }
            }
            false => {
                let expr = input.parse::<LessThanBinaryExpr>()?.expr;
                Self::AlreadyComponent(expr)
            }
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
            Self::Component(component) => {
                quote! { ::oelung::Component::Component(::std::boxed::Box::new(#component)) }
            }
            Self::AlreadyComponent(component) => {
                quote! { #component }
            }
        }
        .to_tokens(tokens)
    }
}

struct FlexColumn {
    pub children: Vec<Element>,
    pub flex_grow: Option<LitFloatOrInt>,
    pub cursor: Option<Cursor>,
}

impl Parse for FlexColumn {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut children: Option<Vec<Element>> = _d();
        let mut flex_grow: Option<LitFloatOrInt> = _d();
        let mut cursor: Option<Cursor> = _d();

        let me_percent_sign_start_column = illicit::expect::<MePercentSignStartColumn>();
        let me_percent_sign_row = illicit::expect::<MePercentSignRow>();
        let parent_percent_sign_start_column = illicit::get::<ParentPercentSignStartColumn>().ok();
        let parent_enclosing_attribute_start_column =
            illicit::get::<ParentEnclosingAttributeStartColumn>().ok();
        illicit::hide::<MePercentSignStartColumn>();
        illicit::hide::<MePercentSignRow>();
        illicit::hide::<ParentEnclosingAttributeStartColumn>();
        let smallest_allowed_column = parent_enclosing_attribute_start_column
            .as_ref()
            .map(|parent_enclosing_attribute_start_column| {
                parent_enclosing_attribute_start_column.0
            })
            .or_else(|| {
                parent_percent_sign_start_column
                    .as_ref()
                    .map(|parent_percent_sign_start_column| parent_percent_sign_start_column.0)
            })
            .unwrap_or(me_percent_sign_start_column.0);
        illicit::Layer::new()
            .offer(ParentPercentSignStartColumn(me_percent_sign_start_column.0))
            .enter(|| -> Result<()> {
                while input.peek(Ident) && input.span().start().column >= smallest_allowed_column {
                    let mut parse_attribute = || -> Result<()> {
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
                            "cursor" => {
                                assert!(cursor.is_none(), "Already saw 'cursor' key");
                                cursor = Some(input.parse()?);
                            }
                            key => return Err(input.error(format!("Unexpected key `{key}`"))),
                        }

                        Ok(())
                    };
                    if input.span().start().line > me_percent_sign_row.0 {
                        illicit::Layer::new()
                            .offer(ParentEnclosingAttributeStartColumn(
                                input.span().start().column,
                            ))
                            .enter(parse_attribute)
                    } else {
                        parse_attribute()
                    }?
                }

                Ok(())
            })?;

        Ok(Self {
            children: children.expect("Expected `children`"),
            flex_grow,
            cursor,
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
        let cursor = match self.cursor.as_ref() {
            None => quote! {},
            Some(cursor) => quote! {
                .cursor(#cursor)
            },
        };

        quote! {
            ::oelung::FlexColumnBuilder::default()
                #(#children)*
                #flex_grow
                #cursor
                .build()?
        }
        .to_tokens(tokens)
    }
}

struct Cursor {
    pub x: LitIntOrExpr,
    pub y: LitIntOrExpr,
}

impl Parse for Cursor {
    fn parse(input: ParseStream) -> Result<Self> {
        // let me_percent_sign_start_column = input.span().start().column;
        // let me_percent_sign_row = input.span().start().line;
        input.parse::<Token![%]>()?;
        let name: Ident = input.parse().unwrap();
        if name.to_string() != "Cursor" {
            return Err(input.error(format!("Expected `Cursor`")));
        }
        input.parse::<Token![.]>()?;
        let relative: Ident = input.parse()?;
        if relative.to_string() != "Relative" {
            return Err(input.error(format!("Expected `Relative`")));
        }

        let mut x: Option<LitIntOrExpr> = _d();
        let mut y: Option<LitIntOrExpr> = _d();

        let parent_percent_sign_start_column = illicit::expect::<ParentPercentSignStartColumn>();
        let parent_enclosing_attribute_start_column =
            illicit::get::<ParentEnclosingAttributeStartColumn>().ok();
        illicit::hide::<ParentPercentSignStartColumn>();
        illicit::hide::<ParentEnclosingAttributeStartColumn>();
        let smallest_allowed_column = parent_enclosing_attribute_start_column
            .as_ref()
            .map(|parent_enclosing_attribute_start_column| {
                parent_enclosing_attribute_start_column.0
            })
            .unwrap_or_else(|| parent_percent_sign_start_column.0);
        println!("next span pre: {:#?}", input.span().start());
        while input.peek(Ident) && input.span().start().column >= smallest_allowed_column {
            println!("next span: {:#?}", input.span().start());
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
    pub children: Vec<TextChild>,
    pub cursor: Option<Cursor>,
    pub color: Option<Color>,
    pub background_color: Option<Color>,
}

impl Parse for Text {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut text: Option<LitStrOrExpr> = _d();
        let mut cursor: Option<Cursor> = _d();
        let mut children: Option<Vec<TextChild>> = _d();
        let mut color: Option<Color> = _d();
        let mut background_color: Option<Color> = _d();

        let me_percent_sign_start_column = illicit::expect::<MePercentSignStartColumn>();
        let me_percent_sign_row = illicit::expect::<MePercentSignRow>();
        let parent_percent_sign_start_column = illicit::get::<ParentPercentSignStartColumn>().ok();
        let parent_enclosing_attribute_start_column =
            illicit::get::<ParentEnclosingAttributeStartColumn>().ok();
        illicit::hide::<MePercentSignStartColumn>();
        illicit::hide::<MePercentSignRow>();
        illicit::hide::<ParentEnclosingAttributeStartColumn>();
        let smallest_allowed_column = parent_enclosing_attribute_start_column
            .as_ref()
            .map(|parent_enclosing_attribute_start_column| {
                parent_enclosing_attribute_start_column.0
            })
            .or_else(|| {
                parent_percent_sign_start_column
                    .as_ref()
                    .map(|parent_percent_sign_start_column| parent_percent_sign_start_column.0)
            })
            .unwrap_or(me_percent_sign_start_column.0);
        illicit::Layer::new()
            .offer(ParentPercentSignStartColumn(me_percent_sign_start_column.0))
            .enter(|| -> Result<()> {
                match input.peek(Ident) && input.peek2(Token![=>]) {
                    true => {
                        while input.peek(Ident)
                            && input.span().start().column >= smallest_allowed_column
                        {
                            let mut parse_attribute = || -> Result<()> {
                                let key = input.parse::<Ident>().unwrap().to_string();
                                input.parse::<Token![=>]>()?;
                                match &*key {
                                    "text" => {
                                        assert!(text.is_none(), "Already saw 'text' key");
                                        assert!(
                                            children.is_none(),
                                            "Only provide one of 'text' or 'children'"
                                        );
                                        text = Some(input.parse()?);
                                    }
                                    "cursor" => {
                                        assert!(cursor.is_none(), "Already saw 'cursor' key");
                                        cursor = Some(input.parse()?);
                                    }
                                    "children" => {
                                        assert!(children.is_none(), "Already saw 'children' key");
                                        assert!(
                                            text.is_none(),
                                            "Only provide one of 'text' or 'children'"
                                        );
                                        let children_content;
                                        bracketed!(children_content in input);
                                        let children = children.populate_default();
                                        while !children_content.is_empty() {
                                            children.push(children_content.parse()?);
                                            children_content.parse::<Option<Token![,]>>()?;
                                        }
                                    }
                                    "color" => {
                                        assert!(color.is_none(), "Already saw 'color' key");
                                        color = Some(input.parse()?);
                                    }
                                    "background_color" => {
                                        assert!(
                                            background_color.is_none(),
                                            "Already saw 'background_color' key"
                                        );
                                        background_color = Some(input.parse()?);
                                    }
                                    key => {
                                        return Err(input.error(format!("Unexpected key `{key}`")))
                                    }
                                }

                                Ok(())
                            };
                            if input.span().start().line > me_percent_sign_row.0 {
                                illicit::Layer::new()
                                    .offer(ParentEnclosingAttributeStartColumn(
                                        input.span().start().column,
                                    ))
                                    .enter(parse_attribute)
                            } else {
                                parse_attribute()
                            }?
                        }
                    }
                    false => {
                        text = Some(input.parse()?);
                    }
                }

                if children.is_none() {
                    children = Some(vec![TextChild::Text(text.expect("Expected `text`"))]);
                }

                Ok(())
            })?;

        Ok(Self {
            children: children.unwrap(),
            cursor,
            color,
            background_color,
        })
    }
}

impl ToTokens for Text {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let children = self.children.iter().map(|child| match child {
            TextChild::Text(text) => quote! { .text_child(#text) },
            TextChild::Nested(nested) => quote! { .nested_child(#nested) },
            TextChild::NestedComponent(nested_component) => {
                quote! { .nested_component_child(::std::boxed::Box::new(#nested_component)) }
            }
        });

        let cursor = match self.cursor.as_ref() {
            None => quote! {},
            Some(cursor) => quote! { .cursor(#cursor) },
        };

        let color = match self.color.as_ref() {
            None => quote! {},
            Some(color) => quote! { .color(#color) },
        };

        let background_color = match self.background_color.as_ref() {
            None => quote! {},
            Some(background_color) => quote! { .background_color(#background_color) },
        };

        quote! {
            ::oelung::TextBuilder::default()
                #(#children)*
                #cursor
                #color
                #background_color
                .build()?
        }
        .to_tokens(tokens)
    }
}

enum TextChild {
    Text(LitStrOrExpr),
    Nested(Text),
    NestedComponent(Expr),
}

impl TextChild {
    pub fn into_text(self) -> LitStrOrExpr {
        match self {
            Self::Text(text) => text,
            _ => panic!("Expected text"),
        }
    }
}

impl Parse for TextChild {
    fn parse(input: ParseStream) -> Result<Self> {
        let element: Element = input.parse()?;

        Ok(match element {
            Element::Text(mut text) => {
                if text.children.len() == 1 && matches!(&text.children[0], TextChild::Text(_)) {
                    Self::Text(text.children.remove(0).into_text())
                } else {
                    Self::Nested(text)
                }
            }
            Element::Component(component) => Self::NestedComponent(component),
            _ => return Err(input.error("Expected text child")),
        })
    }
}

enum Color {
    Ansi(LitInt),
}

impl Parse for Color {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        if name.to_string() != "Ansi" {
            return Err(input.error(format!("Expected 'Ansi'")));
        }
        let color_content;
        parenthesized!(color_content in input);
        let ansi: LitInt = color_content.parse()?;

        Ok(Self::Ansi(ansi))
    }
}

impl ToTokens for Color {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Ansi(ansi) => quote! {
                ::oelung::crossterm::style::Color::AnsiValue(#ansi)
            },
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

enum LitStrOrExpr {
    LitStr(LitStr),
    Expr(Expr),
}

impl Parse for LitStrOrExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        match input.peek(LitStr) {
            true => Ok(Self::LitStr(input.parse().unwrap())),
            false => Ok(Self::Expr(input.parse::<LessThanBinaryExpr>()?.expr)),
        }
    }
}

impl ToTokens for LitStrOrExpr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::LitStr(str) => quote! { #str },
            Self::Expr(expr) => quote! { #expr },
        }
        .to_tokens(tokens)
    }
}

enum LitIntOrExpr {
    LitInt(LitInt),
    Expr(Expr),
}

impl Parse for LitIntOrExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        match input.peek(LitInt) {
            true => Ok(Self::LitInt(input.parse().unwrap())),
            false => Ok(Self::Expr(input.parse::<LessThanBinaryExpr>()?.expr)),
        }
    }
}

impl ToTokens for LitIntOrExpr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::LitInt(int) => quote! { #int },
            Self::Expr(expr) => quote! { #expr },
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

#[derive(Debug)]
struct MePercentSignRow(usize);

#[derive(Debug)]
struct MePercentSignStartColumn(usize);

#[derive(Debug)]
struct ParentPercentSignStartColumn(usize);

#[derive(Debug)]
struct ParentEnclosingAttributeStartColumn(usize);
