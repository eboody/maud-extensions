// JS token lowering: turns js! token input into JavaScript source text.
use proc_macro2::{Delimiter, TokenStream as TokenStream2, TokenTree};

pub(crate) fn tokens_to_source(tokens: TokenStream2) -> String {
    let mut out = String::new();
    let mut prev_word = false;

    for token in tokens {
        match token {
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
                out.push_str(&tokens_to_source(group.stream()));
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
    }

    out
}
