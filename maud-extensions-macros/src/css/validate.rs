// CSS validity checks: structural balance plus lightweight stylesheet parsing.
use cssparser::{BasicParseErrorKind, SourceLocation};

pub(crate) enum StylesheetError {
    UnterminatedComment,
    UnmatchedClosing {
        delimiter: char,
    },
    UnterminatedString,
    UnclosedDelimiter {
        delimiter: char,
    },
    ParserRejectedTokens {
        location: SourceLocation,
        message: String,
    },
}

pub(crate) fn stylesheet(css: &str) -> core::result::Result<(), StylesheetError> {
    validate_structure(css)?;
    let mut input = cssparser::ParserInput::new(css);
    let mut parser = cssparser::Parser::new(&mut input);
    loop {
        match parser.next_including_whitespace_and_comments() {
            Ok(_) => {}
            Err(err) => match err.kind {
                BasicParseErrorKind::EndOfInput => return Ok(()),
                _ => {
                    return Err(StylesheetError::ParserRejectedTokens {
                        location: err.location,
                        message: err.kind.to_string(),
                    });
                }
            },
        }
    }
}

fn validate_structure(css: &str) -> core::result::Result<(), StylesheetError> {
    let mut chars = css.chars().peekable();
    let mut stack = Vec::new();
    let mut string_delim = None;

    while let Some(ch) = chars.next() {
        if let Some(delim) = string_delim {
            match ch {
                '\\' => {
                    chars.next();
                }
                _ if ch == delim => string_delim = None,
                _ => {}
            }
            continue;
        }

        match ch {
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                let mut terminated = false;
                while let Some(comment_ch) = chars.next() {
                    if comment_ch == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        terminated = true;
                        break;
                    }
                }
                if !terminated {
                    return Err(StylesheetError::UnterminatedComment);
                }
            }
            '"' | '\'' => string_delim = Some(ch),
            '{' | '[' | '(' => stack.push(ch),
            '}' => match stack.pop() {
                Some('{') => {}
                _ => {
                    return Err(StylesheetError::UnmatchedClosing { delimiter: '}' });
                }
            },
            ']' => match stack.pop() {
                Some('[') => {}
                _ => {
                    return Err(StylesheetError::UnmatchedClosing { delimiter: ']' });
                }
            },
            ')' => match stack.pop() {
                Some('(') => {}
                _ => {
                    return Err(StylesheetError::UnmatchedClosing { delimiter: ')' });
                }
            },
            _ => {}
        }
    }

    if string_delim.is_some() {
        return Err(StylesheetError::UnterminatedString);
    }

    if let Some(unclosed) = stack.pop() {
        return Err(StylesheetError::UnclosedDelimiter {
            delimiter: unclosed,
        });
    }
    Ok(())
}
