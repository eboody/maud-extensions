// CSS validity checks: structural balance plus lightweight stylesheet parsing.
pub(crate) fn stylesheet(css: &str) -> core::result::Result<(), String> {
    validate_structure(css)?;
    let mut input = cssparser::ParserInput::new(css);
    let mut parser = cssparser::Parser::new(&mut input);
    loop {
        match parser.next_including_whitespace_and_comments() {
            Ok(_) => {}
            Err(err) => match err.kind {
                cssparser::BasicParseErrorKind::EndOfInput => return Ok(()),
                _ => return Err("inline_css! could not parse CSS tokens".to_string()),
            },
        }
    }
}

fn validate_structure(css: &str) -> core::result::Result<(), String> {
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
                    return Err("inline_css! found an unterminated comment".to_string());
                }
            }
            '"' | '\'' => string_delim = Some(ch),
            '{' | '[' | '(' => stack.push(ch),
            '}' => match stack.pop() {
                Some('{') => {}
                _ => {
                    return Err(
                        "inline_css! found an unmatched closing `}` in the stylesheet".to_string(),
                    );
                }
            },
            ']' => match stack.pop() {
                Some('[') => {}
                _ => {
                    return Err(
                        "inline_css! found an unmatched closing `]` in the stylesheet".to_string(),
                    );
                }
            },
            ')' => match stack.pop() {
                Some('(') => {}
                _ => {
                    return Err(
                        "inline_css! found an unmatched closing `)` in the stylesheet".to_string(),
                    );
                }
            },
            _ => {}
        }
    }

    if string_delim.is_some() {
        return Err("inline_css! found an unterminated string literal".to_string());
    }

    if let Some(unclosed) = stack.pop() {
        return Err(format!(
            "inline_css! found an unclosed `{unclosed}` delimiter in the stylesheet"
        ));
    }
    Ok(())
}
