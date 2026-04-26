// Caller-facing css! input forms and their parsing rules.
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

pub(crate) enum MacroInput {
    Inline(Input),
    Named { helper_name: Ident, css: Input },
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(syn::Ident) && input.peek2(Token![,]) {
            let helper_name: Ident = input.parse()?;
            input.parse::<Token![,]>()?;
            let css: Input = input.parse()?;
            if !input.is_empty() {
                return Err(input.error("unexpected trailing tokens after named css! helper"));
            }
            Ok(Self::Named { helper_name, css })
        } else {
            Ok(Self::Inline(input.parse()?))
        }
    }
}
