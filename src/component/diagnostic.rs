// Component derive diagnostics.
use proc_macro2::TokenStream;
use quote::quote;
use syn::Error;

use crate::component::{
    field::{FieldKind, PropField, SlotField},
    model::Component,
};

pub(crate) fn not_yet_implemented(component: &Component) -> TokenStream {
    let name = &component.name;
    let field_count = component.fields.len();
    let generic_count = component.generics.params.len();
    let mut prop_count = 0usize;
    let mut optional_prop_count = 0usize;
    let mut repeated_prop_count = 0usize;
    let mut defaulted_prop_count = 0usize;
    let mut each_prop_count = 0usize;
    let mut slot_count = 0usize;
    let mut optional_slot_count = 0usize;
    let mut repeated_slot_count = 0usize;
    let mut default_slot_count = 0usize;
    let mut each_slot_count = 0usize;

    for field in &component.fields {
        match &field.kind {
            FieldKind::Prop(PropField {
                optional,
                repeated,
                default,
                each,
            }) => {
                prop_count += 1;
                optional_prop_count += usize::from(*optional);
                repeated_prop_count += usize::from(*repeated);
                defaulted_prop_count += usize::from(default.is_some());
                each_prop_count += usize::from(each.is_some());
            }
            FieldKind::Slot(SlotField {
                optional,
                repeated,
                default,
                each,
            }) => {
                slot_count += 1;
                optional_slot_count += usize::from(*optional);
                repeated_slot_count += usize::from(*repeated);
                default_slot_count += usize::from(*default);
                each_slot_count += usize::from(each.is_some());
            }
        }
    }

    syn::Error::new(
        name.span(),
        format!(
            "#[derive(Component)] is not implemented yet; parsed component `{name}` with {field_count} fields, {generic_count} generics, {prop_count} props ({optional_prop_count} optional, {repeated_prop_count} repeated, {defaulted_prop_count} defaulted, {each_prop_count} each) and {slot_count} slots ({optional_slot_count} optional, {repeated_slot_count} repeated, {default_slot_count} default, {each_slot_count} each)"
        ),
    )
    .to_compile_error()
}

pub(crate) fn from_model_error(err: Error) -> TokenStream {
    let rewritten = match err.to_string().as_str() {
        "component-only-structs" => Error::new(
            err.span(),
            "#[derive(Component)] currently only supports structs",
        ),
        "component-only-named-fields" => Error::new(
            err.span(),
            "#[derive(Component)] currently only supports structs with named fields",
        ),
        _ => err,
    };

    rewritten.to_compile_error()
}

pub(crate) fn tokens(ts: TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(quote! { #ts })
}
