// Impl-block component macro rewriting render/css/js item macros into hidden hooks.
use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::{Error, ImplItem, ImplItemMacro, ItemImpl, spanned::Spanned};

pub(crate) fn expand(input: ItemImpl) -> TokenStream {
    match expand_impl(input) {
        Ok(tokens) => TokenStream::from(tokens),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

fn expand_impl(mut input: ItemImpl) -> syn::Result<TokenStream2> {
    let self_ty = (*input.self_ty).clone();
    let mut render_tokens = None;
    let mut css_tokens = None;
    let mut js_tokens = None;
    let mut retained_items = Vec::new();

    for item in input.items {
        match item {
            ImplItem::Macro(item_macro) => match macro_name(&item_macro)? {
                Some("render") => {
                    ensure_unique(&render_tokens, item_macro.mac.path.span(), "render")?;
                    render_tokens = Some(parse_render_tokens(&item_macro)?);
                }
                Some("css") => {
                    ensure_unique(&css_tokens, item_macro.mac.path.span(), "css")?;
                    css_tokens = Some(parse_css_tokens(&item_macro)?);
                }
                Some("js") => {
                    ensure_unique(&js_tokens, item_macro.mac.path.span(), "js")?;
                    js_tokens = Some(parse_js_tokens(&item_macro)?);
                }
                _ => retained_items.push(ImplItem::Macro(item_macro)),
            },
            other => retained_items.push(other),
        }
    }

    let render_tokens = render_tokens.ok_or_else(|| {
        Error::new(
            self_ty.span(),
            "#[mx::component] impls require exactly one `render! { ... }` block",
        )
    })?;

    let css_tokens = css_tokens.unwrap_or_else(|| quote! { ::maud::PreEscaped(::std::string::String::new()) });
    let js_tokens = js_tokens.unwrap_or_else(|| quote! { ::maud::PreEscaped(::std::string::String::new()) });

    input.items = retained_items;

    Ok(quote! {
        #input

        impl #self_ty {
            fn __mx_css() -> ::maud::Markup {
                #css_tokens
            }

            fn __mx_js() -> ::maud::Markup {
                #js_tokens
            }
        }

        impl ::maud_extensions::ComponentRender for #self_ty {
            fn __mx_render(&self) -> ::maud::Markup {
                #render_tokens
            }
        }
    })
}

fn macro_name(item_macro: &ImplItemMacro) -> syn::Result<Option<&'static str>> {
    let Some(ident) = item_macro.mac.path.get_ident() else {
        return Ok(None);
    };

    Ok(match ident.to_string().as_str() {
        "render" => Some("render"),
        "css" => Some("css"),
        "js" => Some("js"),
        _ => None,
    })
}

fn ensure_unique(existing: &Option<TokenStream2>, span: proc_macro2::Span, name: &str) -> syn::Result<()> {
    if existing.is_some() {
        return Err(Error::new(
            span,
            format!("#[mx::component] impls allow at most one `{name}!` block"),
        ));
    }
    Ok(())
}

fn parse_render_tokens(item_macro: &ImplItemMacro) -> syn::Result<TokenStream2> {
    let body = item_macro.mac.tokens.clone();
    let rooted = inject_component_facets(body, item_macro.mac.path.span())?;
    Ok(quote! { ::maud::html! { #rooted } })
}

fn parse_css_tokens(item_macro: &ImplItemMacro) -> syn::Result<TokenStream2> {
    let body = item_macro.mac.tokens.clone();
    Ok(quote! {
        ::maud_extensions::css! {
            #body
        }
    })
}

fn parse_js_tokens(item_macro: &ImplItemMacro) -> syn::Result<TokenStream2> {
    let tokens = item_macro.mac.tokens.clone();
    if tokens.is_empty() {
        return Ok(quote! { ::maud_extensions::js! {} });
    }

    let trees = tokens.clone().into_iter().collect::<Vec<_>>();
    if trees.len() >= 2
        && matches!(&trees[0], TokenTree::Ident(_))
        && matches!(&trees[1], TokenTree::Punct(punct) if punct.as_char() == ',')
    {
        let TokenTree::Ident(mode) = &trees[0] else {
            unreachable!();
        };

        if mode != "once" {
            return Err(Error::new(
                mode.span(),
                "`js!` in #[mx::component] impls only supports `js! { ... }` or `js!(once, { ... })`",
            ));
        }

        return Ok(quote! { ::maud_extensions::js!(#tokens) });
    }

    Ok(quote! { ::maud_extensions::js! { #tokens } })
}

fn inject_component_facets(
    tokens: TokenStream2,
    span: proc_macro2::Span,
) -> syn::Result<TokenStream2> {
    let trees = tokens.into_iter().collect::<Vec<_>>();
    let root_index = trees
        .iter()
        .enumerate()
        .filter(|(_, tree)| matches!(tree, TokenTree::Group(group) if group.delimiter() == Delimiter::Brace))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();

    if root_index.len() != 1 {
        return Err(Error::new(
            span,
            "`render!` must contain exactly one top-level element with one trailing `{ ... }` body block",
        ));
    }

    let body_index = root_index[0];
    if body_index == 0 || body_index != trees.len() - 1 {
        return Err(Error::new(
            span,
            "`render!` must look like one root element followed by one `{ ... }` body block",
        ));
    }

    let mut output = TokenStream2::new();
    for tree in &trees[..body_index] {
        output.extend([tree.clone()]);
    }

    let TokenTree::Group(body_group) = &trees[body_index] else {
        unreachable!();
    };
    let original_body = body_group.stream();
    let expanded_body = quote! {
        (Self::__mx_css())
        (Self::__mx_js())
        #original_body
    };
    let mut new_group = Group::new(Delimiter::Brace, expanded_body);
    new_group.set_span(body_group.span());
    output.extend([TokenTree::Group(new_group)]);

    Ok(output)
}
