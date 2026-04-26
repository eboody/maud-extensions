// CSS helper DSL: recognizes and expands raw!/at-rule/unit helper forms.
use proc_macro2::{Delimiter, Group, TokenTree};
use syn::{
    LitStr, Result,
    parse::{Parse, ParseStream},
};

use crate::css::source;

pub(crate) fn try_expand(tokens: &[TokenTree], index: &mut usize) -> Result<Option<String>> {
    if let Some(raw_css) = try_parse_raw_fragment(tokens, index)? {
        return Ok(Some(raw_css));
    }

    for (macro_name, at_rule) in [
        ("media", "media"),
        ("container", "container"),
        ("supports", "supports"),
        ("layer", "layer"),
        ("keyframes", "keyframes"),
    ] {
        let original_index = *index;
        if let Some(css) = try_parse_at_rule(tokens, index, macro_name, at_rule)? {
            return Ok(Some(css));
        }
        *index = original_index;
    }

    if let Some(unit_value) = try_parse_unit(tokens, index)? {
        return Ok(Some(unit_value));
    }

    Ok(None)
}

pub(crate) fn ends_with_word(css: &str) -> bool {
    css.chars()
        .next_back()
        .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '%' || ch == '-')
}

struct RawFragment {
    css: LitStr,
}

impl Parse for RawFragment {
    fn parse(input: ParseStream) -> Result<Self> {
        let css: LitStr = input.parse()?;
        if !input.is_empty() {
            return Err(input.error("raw! expects exactly one string literal argument"));
        }
        Ok(Self { css })
    }
}

fn parse_raw_fragment(group: &Group) -> Result<String> {
    let input = syn::parse2::<RawFragment>(group.stream()).map_err(|_| {
        syn::Error::new(
            group.span(),
            "raw! expects exactly one string literal argument",
        )
    })?;
    Ok(input.css.value())
}

fn parse_macro_group<'a>(
    tokens: &'a [TokenTree],
    index: &mut usize,
    macro_name: &str,
) -> Result<Option<&'a Group>> {
    let Some(TokenTree::Ident(ident)) = tokens.get(*index) else {
        return Ok(None);
    };
    if ident != macro_name {
        return Ok(None);
    }
    let Some(TokenTree::Punct(punct)) = tokens.get(*index + 1) else {
        return Ok(None);
    };
    if punct.as_char() != '!' {
        return Ok(None);
    }
    let Some(TokenTree::Group(group)) = tokens.get(*index + 2) else {
        return Err(syn::Error::new(
            punct.span(),
            format!("{macro_name}! expects macro arguments in parentheses"),
        ));
    };
    if group.delimiter() != Delimiter::Parenthesis {
        return Err(syn::Error::new(
            group.span(),
            format!("{macro_name}! expects macro arguments in parentheses"),
        ));
    }
    *index += 2;
    Ok(Some(group))
}

fn try_parse_raw_fragment(tokens: &[TokenTree], index: &mut usize) -> Result<Option<String>> {
    let Some(group) = parse_macro_group(tokens, index, "raw")? else {
        return Ok(None);
    };
    Ok(Some(parse_raw_fragment(group)?))
}

fn try_parse_at_rule(
    tokens: &[TokenTree],
    index: &mut usize,
    macro_name: &str,
    at_rule: &str,
) -> Result<Option<String>> {
    let Some(group) = parse_macro_group(tokens, index, macro_name)? else {
        return Ok(None);
    };
    Ok(Some(parse_at_rule(group, macro_name, at_rule)?))
}

fn parse_at_rule(group: &Group, macro_name: &str, at_rule: &str) -> Result<String> {
    let tokens: Vec<TokenTree> = group.stream().into_iter().collect();
    let Some(body_index) = tokens.iter().rposition(
        |token| matches!(token, TokenTree::Group(group) if group.delimiter() == Delimiter::Brace),
    ) else {
        return Err(syn::Error::new(
            group.span(),
            format!("{macro_name}! expects a prelude and trailing `{{ ... }}` body"),
        ));
    };

    if body_index < 2
        || !matches!(tokens.get(body_index - 1), Some(TokenTree::Punct(punct)) if punct.as_char() == ',')
    {
        return Err(syn::Error::new(
            group.span(),
            format!("{macro_name}! expects `{macro_name}!(prelude, {{ ... }})`"),
        ));
    }

    if tokens
        .iter()
        .skip(body_index + 1)
        .any(|token| !matches!(token, TokenTree::Punct(punct) if punct.as_char() == ','))
    {
        return Err(syn::Error::new(
            group.span(),
            format!("{macro_name}! only accepts a prelude and one body block"),
        ));
    }

    let prelude_tokens = &tokens[..body_index - 1];
    let prelude = if let [TokenTree::Literal(literal)] = prelude_tokens {
        match syn::parse_str::<LitStr>(&literal.to_string()) {
            Ok(lit) => lit.value(),
            Err(_) => literal.to_string(),
        }
    } else {
        source::token_trees_to_source(prelude_tokens.to_vec())?
    };
    if prelude.trim().is_empty() {
        return Err(syn::Error::new(
            group.span(),
            format!("{macro_name}! requires a non-empty prelude"),
        ));
    }
    let TokenTree::Group(body_group) = &tokens[body_index] else {
        unreachable!();
    };
    let body = source::tokens_to_source(body_group.stream())?;
    Ok(format!("@{at_rule} {prelude} {{{body}}}"))
}

fn try_parse_unit(tokens: &[TokenTree], index: &mut usize) -> Result<Option<String>> {
    for (macro_name, suffix) in [
        ("rem", "rem"),
        ("em", "em"),
        ("px", "px"),
        ("pct", "%"),
        ("vw", "vw"),
        ("vh", "vh"),
        ("ms", "ms"),
        ("s", "s"),
    ] {
        let original_index = *index;
        if let Some(group) = parse_macro_group(tokens, index, macro_name)? {
            return Ok(Some(parse_unit(group, macro_name, suffix)?));
        }
        *index = original_index;
    }
    Ok(None)
}

fn parse_unit(group: &Group, macro_name: &str, suffix: &str) -> Result<String> {
    let tokens: Vec<TokenTree> = group.stream().into_iter().collect();
    if tokens.is_empty() {
        return Err(syn::Error::new(
            group.span(),
            format!("{macro_name}! requires a value"),
        ));
    }
    if tokens
        .iter()
        .any(|token| matches!(token, TokenTree::Punct(punct) if punct.as_char() == ','))
    {
        return Err(syn::Error::new(
            group.span(),
            format!("{macro_name}! expects exactly one value argument"),
        ));
    }
    let value = source::token_trees_to_source(tokens)?;
    if value.trim().is_empty() {
        return Err(syn::Error::new(
            group.span(),
            format!("{macro_name}! requires a value"),
        ));
    }
    Ok(format!("{value}{suffix}"))
}
