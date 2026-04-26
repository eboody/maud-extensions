// Caller-facing js! input forms, including once-mode parsing.
use proc_macro2::{Ident, TokenStream as TokenStream2, TokenTree};
use syn::spanned::Spanned;
use syn::{
    LitStr, Result, Token, braced,
    parse::{Parse, ParseStream},
};

use crate::js::diagnostics;

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

#[derive(Clone, Copy)]
pub(crate) enum Mode {
    Always,
    Once,
}

pub(crate) enum MacroInput {
    Inline {
        mode: Mode,
        js: Input,
    },
    Named {
        helper_name: Ident,
        mode: Mode,
        js: Input,
    },
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
            let first: Ident = input.parse()?;
            input.parse::<Token![,]>()?;

            if first == "once" {
                let js = parse_inline_input(input)?;
                return Ok(Self::Inline {
                    mode: Mode::Once,
                    js,
                });
            }

            if input.peek(syn::Ident) && input.peek2(Token![,]) {
                let mode_ident: Ident = input.parse()?;
                input.parse::<Token![,]>()?;
                let mode = if mode_ident == "once" {
                    Mode::Once
                } else {
                    return Err(diagnostics::named_helper_mode_must_be_once(
                        mode_ident.span(),
                    ));
                };
                let js = parse_named_input(input)?;
                return Ok(Self::Named {
                    helper_name: first,
                    mode,
                    js,
                });
            }

            let js = parse_named_input(input)?;
            Ok(Self::Named {
                helper_name: first,
                mode: Mode::Always,
                js,
            })
        } else {
            Ok(Self::Inline {
                mode: Mode::Always,
                js: parse_inline_input(input)?,
            })
        }
    }
}

fn parse_inline_input(input: ParseStream) -> Result<Input> {
    let js: Input = input.parse()?;
    if !input.is_empty() {
        let trailing_span = next_trailing_span(input)?;
        return Err(diagnostics::unexpected_trailing_tokens_after_body(
            trailing_span,
        ));
    }
    Ok(js)
}

fn parse_named_input(input: ParseStream) -> Result<Input> {
    let js = if input.peek(LitStr) {
        Input::Literal(input.parse()?)
    } else if input.peek(syn::token::Brace) {
        let content;
        braced!(content in input);
        Input::Tokens(content.parse()?)
    } else {
        Input::Tokens(input.parse()?)
    };

    if !input.is_empty() {
        let trailing_span = next_trailing_span(input)?;
        return Err(diagnostics::unexpected_trailing_tokens_after_named_helper(
            trailing_span,
        ));
    }

    Ok(js)
}

fn next_trailing_span(input: ParseStream) -> Result<proc_macro2::Span> {
    if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
        if input.is_empty() {
            Ok(input.span())
        } else {
            let trailing: TokenTree = input.parse()?;
            Ok(trailing.span())
        }
    } else {
        let trailing: TokenTree = input.parse()?;
        Ok(trailing.span())
    }
}
