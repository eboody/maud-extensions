// Caller-facing js! input forms, including once-mode parsing.
use proc_macro2::{Ident, TokenStream as TokenStream2};
use syn::{
    LitStr, Result, Token,
    parse::{Parse, ParseStream},
};

pub(crate) enum Input {
    Literal(LitStr),
    Tokens(TokenStream2),
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
        if input.peek(syn::Ident) && input.peek2(Token![,]) {
            let first: Ident = input.parse()?;
            input.parse::<Token![,]>()?;

            if first == "once" {
                let js: Input = input.parse()?;
                if !input.is_empty() {
                    return Err(input.error("unexpected trailing tokens after js! body"));
                }
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
                    return Err(syn::Error::new(
                        mode_ident.span(),
                        "js! named helper mode must be `once`.",
                    ));
                };
                let js: Input = input.parse()?;
                if !input.is_empty() {
                    return Err(input.error("unexpected trailing tokens after named js! helper"));
                }
                return Ok(Self::Named {
                    helper_name: first,
                    mode,
                    js,
                });
            }

            let js: Input = input.parse()?;
            if !input.is_empty() {
                return Err(input.error("unexpected trailing tokens after named js! helper"));
            }
            Ok(Self::Named {
                helper_name: first,
                mode: Mode::Always,
                js,
            })
        } else {
            Ok(Self::Inline {
                mode: Mode::Always,
                js: input.parse()?,
            })
        }
    }
}
