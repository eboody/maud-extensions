// Component derive expansion orchestration.
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{GenericParam, Generics};

use crate::component::{
    attrs::DefaultValue,
    diagnostic,
    field::{ComponentField, FieldKind, PropField, SlotField},
    model::Component,
};

pub(crate) fn derive(component: syn::Result<Component>) -> TokenStream {
    match component {
        Ok(component) => match derive_bon_v1(&component) {
            Ok(tokens) => TokenStream::from(tokens),
            Err(tokens) => diagnostic::tokens(tokens),
        },
        Err(err) => diagnostic::tokens(diagnostic::from_model_error(err)),
    }
}

fn derive_bon_v1(component: &Component) -> Result<TokenStream2, TokenStream2> {
    let default_slot = component.default_slot();

    for field in &component.fields {
        match &field.kind {
            FieldKind::Slot(slot) => {
                if slot.optional || slot.repeated || slot.each.is_some() {
                    if Some(&field.name) == default_slot.map(|field| &field.name) {
                        return Err(diagnostic::unsupported_in_v1(
                            component,
                            &field.name.to_string(),
                            "optional or repeated default slot semantics",
                        ));
                    }

                    if !(slot.optional && !slot.repeated && slot.each.is_none()
                        || !slot.optional && slot.repeated && slot.each.is_some())
                    {
                        return Err(diagnostic::unsupported_in_v1(
                            component,
                            &field.name.to_string(),
                            "repeated or advanced named slot semantics",
                        ));
                    }
                }

                if Some(&field.name) == default_slot.map(|field| &field.name) {
                    if !is_maud_markup(&field.ty) {
                        return Err(diagnostic::unsupported_in_v1(
                            component,
                            &field.name.to_string(),
                            "default slots with non-`maud::Markup` storage types",
                        ));
                    }
                } else if slot.repeated {
                    if !is_vec_of_maud_markup(&field.ty) {
                        return Err(diagnostic::unsupported_in_v1(
                            component,
                            &field.name.to_string(),
                            "repeated slots with non-`Vec<maud::Markup>` storage types",
                        ));
                    }
                } else if !is_optional_maud_markup(&field.ty) {
                    return Err(diagnostic::unsupported_in_v1(
                        component,
                        &field.name.to_string(),
                        "named optional slots with non-`Option<maud::Markup>` storage types",
                    ));
                }
            }
            FieldKind::Prop(PropField { each: Some(_), .. }) => {
                return Err(diagnostic::unsupported_in_v1(
                    component,
                    &field.name.to_string(),
                    "`each` setters",
                ));
            }
            FieldKind::Prop(_) => {}
        }
    }

    let name = &component.name;
    let builder_name = format_ident!("__Mx{}Builder", name);
    let state_mod_name = format_ident!("__mx_{}_builder", name.to_string().to_lowercase());
    let (impl_generics, ty_generics, where_clause) = component.generics.split_for_impl();
    let builder_impl_generics = builder_impl_generics(&component.generics, &state_mod_name);
    let builder_type_generics = builder_type_generics(&component.generics);
    let params = component
        .fields
        .iter()
        .map(prop_param)
        .collect::<Result<Vec<_>, _>>()?;
    let slot_methods = default_slot
        .map(|field| {
            default_slot_methods(field, &component.generics, &builder_name, &state_mod_name)
        })
        .transpose()?;
    let named_slot_methods = component
        .named_slots()
        .map(|field| match field.slot() {
            Some(SlotField { repeated: true, .. }) => {
                repeated_slot_methods(field, &component.generics, &builder_name, &state_mod_name)
            }
            _ => named_optional_slot_methods(
                field,
                &component.generics,
                &builder_name,
                &state_mod_name,
            ),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let inits = component
        .fields
        .iter()
        .map(|field| {
            let field_name = &field.name;
            let builder_name = &field.builder_name;
            quote! { #field_name: #builder_name }
        })
        .collect::<Vec<_>>();

    Ok(quote! {
        #[::bon::bon]
        impl #impl_generics #name #ty_generics #where_clause {
            #[builder(
                builder_type(name = #builder_name, vis = "pub"),
                state_mod(name = #state_mod_name, vis = "pub"),
                start_fn(name = new, vis = "pub"),
                finish_fn(name = build, vis = "pub"),
                on(String, into)
            )]
            fn __mx_component_new(
                #( #params ),*
            ) -> Self {
                Self {
                    #( #inits ),*
                }
            }
        }

        impl #builder_impl_generics #builder_name #builder_type_generics {
            #slot_methods
            #( #named_slot_methods )*

            pub fn render(self) -> ::maud::Markup
            where
                S: #state_mod_name::IsComplete,
                #name #ty_generics: ::maud::Render,
            {
                ::maud::Render::render(&self.build())
            }
        }
    })
}

fn prop_param(field: &ComponentField) -> Result<TokenStream2, TokenStream2> {
    let builder_name = &field.builder_name;
    let ty = &field.ty;

    match &field.kind {
        FieldKind::Prop(prop) => {
            let default_attr = match &prop.default {
                Some(DefaultValue {
                    expr: Some(expr), ..
                }) => quote! { #[builder(default = #expr)] },
                Some(DefaultValue { expr: None, .. }) => quote! { #[builder(default)] },
                None => TokenStream2::new(),
            };

            Ok(quote! {
                #default_attr
                #builder_name: #ty
            })
        }
        FieldKind::Slot(SlotField { default: true, .. }) => {
            let internal_name = field.bon_required_internal_setter_ident();

            Ok(quote! {
                #[builder(setters(name = #internal_name, vis = ""))]
                #builder_name: #ty
            })
        }
        FieldKind::Slot(SlotField {
            default: false,
            repeated: true,
            ..
        }) => {
            let getter_internal = format_ident!("__mx_get_{}_internal", builder_name);

            Ok(quote! {
                #[builder(default, overwritable, getter(name = #getter_internal, vis = ""))]
                #builder_name: #ty
            })
        }
        FieldKind::Slot(SlotField { default: false, .. }) => {
            let some_internal = field.bon_optional_some_setter_ident();

            Ok(quote! {
                #[builder(setters(some_fn(name = #some_internal, vis = "")))]
                #builder_name: #ty
            })
        }
    }
}

fn default_slot_methods(
    field: &ComponentField,
    generics: &Generics,
    builder_name: &syn::Ident,
    state_mod_name: &syn::Ident,
) -> Result<TokenStream2, TokenStream2> {
    let slot_name = &field.name;
    let internal_name = field.bon_required_internal_setter_ident();
    let state_assoc = field.state_assoc_ident();
    let set_state = field.set_state_ident();
    let builder_after_slot = builder_type_with_state(
        generics,
        builder_name,
        quote! { #state_mod_name::#set_state<S> },
    );

    Ok(quote! {
        pub fn #slot_name<NewChild>(self, child: NewChild) -> #builder_after_slot
        where
            S::#state_assoc: #state_mod_name::IsUnset,
            NewChild: ::maud::Render,
        {
            self.#internal_name(::maud::Render::render(&child))
        }

        pub fn child<NewChild>(self, child: NewChild) -> #builder_after_slot
        where
            S::#state_assoc: #state_mod_name::IsUnset,
            NewChild: ::maud::Render,
        {
            self.#slot_name(child)
        }
    })
}

fn repeated_slot_methods(
    field: &ComponentField,
    _generics: &Generics,
    _builder_name: &syn::Ident,
    _state_mod_name: &syn::Ident,
) -> Result<TokenStream2, TokenStream2> {
    let item_setter = field
        .slot()
        .and_then(|slot| slot.each.as_ref())
        .expect("repeated slot methods require `each = ...`");
    let collection_setter = &field.name;
    let getter_internal = format_ident!("__mx_get_{}_internal", collection_setter);

    Ok(quote! {
        pub fn #item_setter<NewChild>(self, child: NewChild) -> Self
        where
            NewChild: ::maud::Render,
        {
            let mut items = self.#getter_internal().cloned().unwrap_or_default();
            items.push(::maud::Render::render(&child));
            self.#collection_setter(items)
        }
    })
}

fn named_optional_slot_methods(
    field: &ComponentField,
    generics: &Generics,
    builder_name: &syn::Ident,
    state_mod_name: &syn::Ident,
) -> Result<TokenStream2, TokenStream2> {
    let slot_name = &field.name;
    let some_internal = field.bon_optional_some_setter_ident();
    let state_assoc = field.state_assoc_ident();
    let set_state = field.set_state_ident();
    let builder_after_slot = builder_type_with_state(
        generics,
        builder_name,
        quote! { #state_mod_name::#set_state<S> },
    );

    Ok(quote! {
        pub fn #slot_name<NewChild>(self, child: NewChild) -> #builder_after_slot
        where
            S::#state_assoc: #state_mod_name::IsUnset,
            NewChild: ::maud::Render,
        {
            self.#some_internal(::maud::Render::render(&child))
        }
    })
}

fn is_maud_markup(ty: &syn::Type) -> bool {
    let syn::Type::Path(type_path) = ty else {
        return false;
    };

    let segments = type_path.path.segments.iter().collect::<Vec<_>>();
    let Some(first) = segments.get(segments.len().saturating_sub(2)) else {
        return false;
    };
    let Some(second) = segments.last() else {
        return false;
    };

    first.ident == "maud" && second.ident == "Markup"
}

fn is_optional_maud_markup(ty: &syn::Type) -> bool {
    let syn::Type::Path(type_path) = ty else {
        return false;
    };

    let Some(option_segment) = type_path.path.segments.last() else {
        return false;
    };
    if option_segment.ident != "Option" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &option_segment.arguments else {
        return false;
    };
    let Some(syn::GenericArgument::Type(inner)) = arguments.args.first() else {
        return false;
    };

    is_maud_markup(inner)
}

fn is_vec_of_maud_markup(ty: &syn::Type) -> bool {
    let syn::Type::Path(type_path) = ty else {
        return false;
    };

    let Some(vec_segment) = type_path.path.segments.last() else {
        return false;
    };
    if vec_segment.ident != "Vec" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &vec_segment.arguments else {
        return false;
    };
    let Some(syn::GenericArgument::Type(inner)) = arguments.args.first() else {
        return false;
    };

    is_maud_markup(inner)
}

fn builder_impl_generics(generics: &Generics, state_mod_name: &syn::Ident) -> TokenStream2 {
    let params = generics.params.iter();

    if generics.params.is_empty() {
        quote! { <S: #state_mod_name::State> }
    } else {
        quote! { <#(#params,)* S: #state_mod_name::State> }
    }
}

fn builder_type_generics(generics: &Generics) -> TokenStream2 {
    let args = generics.params.iter().map(generic_arg);

    if generics.params.is_empty() {
        quote! { <S> }
    } else {
        quote! { <#(#args,)* S> }
    }
}

fn builder_type_with_state(
    generics: &Generics,
    builder_name: &syn::Ident,
    state: TokenStream2,
) -> TokenStream2 {
    let args = generics.params.iter().map(generic_arg);

    if generics.params.is_empty() {
        quote! { #builder_name<#state> }
    } else {
        quote! { #builder_name<#(#args,)* #state> }
    }
}

fn generic_arg(param: &GenericParam) -> TokenStream2 {
    match param {
        GenericParam::Lifetime(lifetime) => {
            let lifetime = &lifetime.lifetime;
            quote! { #lifetime }
        }
        GenericParam::Type(ty) => {
            let ident = &ty.ident;
            quote! { #ident }
        }
        GenericParam::Const(_) => unreachable!("const generics are rejected in the model"),
    }
}
