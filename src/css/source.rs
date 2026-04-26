// CSS source rendering: turns token trees into stylesheet text.
use proc_macro2::{Delimiter, TokenStream as TokenStream2, TokenTree};
use syn::Result;

use crate::css::dsl;

pub(crate) fn tokens_to_source(tokens: TokenStream2) -> Result<String> {
    token_trees_to_source(tokens.into_iter().collect())
}

pub(crate) fn token_trees_to_source(tokens: Vec<TokenTree>) -> Result<String> {
    let mut out = String::new();
    let mut prev_word = false;
    let mut index = 0usize;

    while index < tokens.len() {
        if let Some(css) = dsl::try_expand(&tokens, &mut index)? {
            if prev_word {
                out.push(' ');
            }
            out.push_str(&css);
            prev_word = dsl::ends_with_word(&css);
            continue;
        }

        match &tokens[index] {
            TokenTree::Group(group) => {
                let (open, close) = match group.delimiter() {
                    Delimiter::Parenthesis => ('(', ')'),
                    Delimiter::Bracket => ('[', ']'),
                    Delimiter::Brace => ('{', '}'),
                    Delimiter::None => (' ', ' '),
                };
                let needs_space =
                    prev_word && matches!(group.delimiter(), Delimiter::Brace | Delimiter::None);
                if needs_space {
                    out.push(' ');
                }
                if open != ' ' {
                    out.push(open);
                }
                out.push_str(&tokens_to_source(group.stream())?);
                if close != ' ' {
                    out.push(close);
                }
                prev_word = false;
            }
            TokenTree::Ident(ident) => {
                if prev_word {
                    out.push(' ');
                }
                out.push_str(&ident.to_string());
                prev_word = true;
            }
            TokenTree::Literal(literal) => {
                if prev_word {
                    out.push(' ');
                }
                out.push_str(&literal.to_string());
                prev_word = true;
            }
            TokenTree::Punct(punct) => {
                out.push(punct.as_char());
                prev_word = false;
            }
        }

        index += 1;
    }

    Ok(out)
}
