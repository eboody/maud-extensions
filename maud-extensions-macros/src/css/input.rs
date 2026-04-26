// Caller-facing css! input forms and their parsing rules.
use proc_macro2::{Ident, TokenStream as TokenStream2, TokenTree};
use syn::spanned::Spanned;
use syn::{
    LitStr, Result, Token, braced,
    parse::{Parse, ParseStream},
};

use crate::css::diagnostics;

pub(crate) enum Input {
    Literal(LitStr),
    Tokens(TokenStream2),
}

impl Input {
    pub(crate) fn span(&self) -> proc_macro2::Span {
        match self {
            Self::Literal(lit) => lit.span(),
            Self::Tokens(tokens) => tokens.span(),
        }
    }
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(LitStr) {
            Ok(Self::Literal(input.parse()?))
        } else {
            Ok(Self::Tokens(input.parse()?))
        }
    }
}

pub(crate) enum MacroInput {
    Inline(Input),
    Named { helper_name: Ident, css: Input },
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(LitStr) && input.peek2(Token![,]) {
            let helper_name: LitStr = input.parse()?;
            return Err(diagnostics::helper_name_must_be_identifier(
                helper_name.span(),
            ));
        }

        if input.peek(syn::Ident) && input.peek2(Token![,]) {
            let helper_name: Ident = input.parse()?;
            input.parse::<Token![,]>()?;
            let css = parse_named_input(input)?;
            Ok(Self::Named { helper_name, css })
        } else {
            Ok(Self::Inline(input.parse()?))
        }
    }
}

fn parse_named_input(input: ParseStream) -> Result<Input> {
    let css = if input.peek(LitStr) {
        Input::Literal(input.parse()?)
    } else if input.peek(syn::token::Brace) {
        let content;
        braced!(content in input);
        Input::Tokens(content.parse()?)
    } else {
        Input::Tokens(input.parse()?)
    };

    if !input.is_empty() {
        let trailing_span = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                input.span()
            } else {
                let trailing: TokenTree = input.parse()?;
                trailing.span()
            }
        } else {
            let trailing: TokenTree = input.parse()?;
            trailing.span()
        };
        return Err(diagnostics::unexpected_trailing_tokens_after_named_helper(
            trailing_span,
        ));
    }

    Ok(css)
}
