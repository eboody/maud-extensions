// Semantic component model built from the user's struct declaration.
use syn::{Data, DataStruct, Fields, GenericParam, Generics, Ident, Result, spanned::Spanned};

use crate::component::field::ComponentField;
use crate::component::input::Input;

pub(crate) struct Component {
    pub(crate) name: Ident,
    pub(crate) generics: Generics,
    pub(crate) fields: Vec<ComponentField>,
}

impl Component {
    pub(crate) fn from_input(input: Input) -> Result<Self> {
        let data = match input.data {
            Data::Struct(data) => data,
            _ => {
                return Err(syn::Error::new(
                    input.ident.span(),
                    "component-only-structs",
                ));
            }
        };

        let fields = parse_fields(&input.ident, data)?;
        ensure_at_most_one_default_slot(&fields)?;
        ensure_no_const_generics(&input.generics)?;

        Ok(Self {
            name: input.ident,
            generics: input.generics,
            fields,
        })
    }
}

fn parse_fields(component_name: &Ident, data: DataStruct) -> Result<Vec<ComponentField>> {
    let named = match data.fields {
        Fields::Named(named) => named,
        Fields::Unnamed(_) | Fields::Unit => {
            return Err(syn::Error::new(
                component_name.span(),
                "component-only-named-fields",
            ));
        }
    };

    named
        .named
        .into_iter()
        .map(ComponentField::from_syn)
        .collect()
}

fn ensure_at_most_one_default_slot(fields: &[ComponentField]) -> Result<()> {
    let mut default_slot_name: Option<&Ident> = None;

    for field in fields {
        let crate::component::field::FieldKind::Slot(slot) = &field.kind else {
            continue;
        };

        if !slot.default {
            continue;
        }

        if let Some(existing) = default_slot_name {
            return Err(syn::Error::new(
                field.name.span(),
                format!(
                    "component allows at most one default slot; `{}` is already the default slot",
                    existing
                ),
            ));
        }

        default_slot_name = Some(&field.name);
    }

    Ok(())
}

fn ensure_no_const_generics(generics: &Generics) -> Result<()> {
    for generic in &generics.params {
        if let GenericParam::Const(const_generic) = generic {
            return Err(syn::Error::new(
                const_generic.span(),
                "#[derive(Component)] does not support const generics yet",
            ));
        }
    }

    Ok(())
}
