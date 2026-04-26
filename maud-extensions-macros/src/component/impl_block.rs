// Impl-block component macro rewriting render/css/js item macros into hidden hooks.
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Error, ImplItem, ImplItemMacro, ItemImpl, MacroDelimiter, Token,
    parse::ParseStream,
    spanned::Spanned,
};

pub(crate) fn expand(mut input: ItemImpl) -> TokenStream {
    let mut seen_render = false;
    let mut seen_css = false;
    let mut seen_js = false;
    let mut errors = Vec::new();

    for item in &mut input.items {
        let ImplItem::Macro(item_macro) = item else {
            continue;
        };

        match rewrite_macro_item(item_macro, &mut seen_render, &mut seen_css, &mut seen_js) {
            Ok(new_item) => *item = new_item,
            Err(err) => errors.push(err),
        }
    }

    if let Some(err) = errors.into_iter().reduce(|mut left, right| {
        left.combine(right);
        left
    }) {
        return TokenStream::from(err.to_compile_error());
    }

    TokenStream::from(quote! { #input })
}

fn rewrite_macro_item(
    item_macro: &ImplItemMacro,
    seen_render: &mut bool,
    seen_css: &mut bool,
    seen_js: &mut bool,
) -> syn::Result<ImplItem> {
    let Some(ident) = item_macro.mac.path.get_ident() else {
        return Ok(ImplItem::Macro(item_macro.clone()));
    };

    match ident.to_string().as_str() {
        "render" => {
            if *seen_render {
                return Err(Error::new(
                    item_macro.mac.path.span(),
                    "#[mx::component] allows at most one `render! { ... }` block per impl",
                ));
            }
            *seen_render = true;
            Ok(parse_render_item(item_macro))
        }
        "css" => {
            if *seen_css {
                return Err(Error::new(
                    item_macro.mac.path.span(),
                    "#[mx::component] allows at most one `css! { ... }` block per impl",
                ));
            }
            *seen_css = true;
            Ok(parse_css_item(item_macro))
        }
        "js" => {
            if *seen_js {
                return Err(Error::new(
                    item_macro.mac.path.span(),
                    "#[mx::component] allows at most one `js! ...` block per impl",
                ));
            }
            *seen_js = true;
            parse_js_item(item_macro)
        }
        _ => Ok(ImplItem::Macro(item_macro.clone())),
    }
}

fn parse_render_item(item_macro: &ImplItemMacro) -> ImplItem {
    let body = &item_macro.mac.tokens;
    syn::parse_quote! {
        fn __mx_render(&self) -> ::maud::Markup {
            ::maud::html! { #body }
        }
    }
}

fn parse_css_item(item_macro: &ImplItemMacro) -> ImplItem {
    let body = &item_macro.mac.tokens;
    syn::parse_quote! {
        fn __mx_css() -> ::maud::Markup {
            ::maud_extensions::css! { #body }
        }
    }
}

fn parse_js_item(item_macro: &ImplItemMacro) -> syn::Result<ImplItem> {
    let mode_once = parse_js_mode(&item_macro.mac)?;
    let body = js_body_tokens(&item_macro.mac)?;

    Ok(if mode_once {
        syn::parse_quote! {
            fn __mx_js() -> ::maud::Markup {
                ::maud_extensions::js!(once, { #body })
            }
        }
    } else {
        syn::parse_quote! {
            fn __mx_js() -> ::maud::Markup {
                ::maud_extensions::js! { #body }
            }
        }
    })
}

fn parse_js_mode(mac: &syn::Macro) -> syn::Result<bool> {
    match mac.delimiter {
        MacroDelimiter::Brace(_) => Ok(false),
        MacroDelimiter::Paren(_) => syn::parse::Parser::parse2(
            |input: ParseStream| {
                let ident: syn::Ident = input.parse()?;
                if ident != "once" {
                    return Err(Error::new(
                        ident.span(),
                        "`js!` in #[mx::component] impls only supports `js! { ... }` or `js!(once, { ... })`",
                    ));
                }
                input.parse::<Token![,]>()?;
                Ok(true)
            },
            mac.tokens.clone(),
        ),
        _ => Err(Error::new(
            mac.delimiter.span().open(),
            "`js!` in #[mx::component] impls only supports brace-delimited or parenthesized forms",
        )),
    }
}

fn js_body_tokens(mac: &syn::Macro) -> syn::Result<TokenStream2> {
    match mac.delimiter {
        MacroDelimiter::Brace(_) => Ok(mac.tokens.clone()),
        MacroDelimiter::Paren(_) => syn::parse::Parser::parse2(
            |input: ParseStream| {
                let _mode: syn::Ident = input.parse()?;
                input.parse::<Token![,]>()?;
                let group: syn::Block = input.parse()?;
                let stmts = group.stmts;
                Ok(quote! { #(#stmts)* })
            },
            mac.tokens.clone(),
        )
        .map_err(|_| {
            Error::new(
                mac.tokens.span(),
                "`js!` in #[mx::component] impls only supports `js! { ... }` or `js!(once, { ... })`",
            )
        }),
        _ => Err(Error::new(
            mac.delimiter.span().open(),
            "`js!` in #[mx::component] impls only supports brace-delimited or parenthesized forms",
        )),
    }
}
