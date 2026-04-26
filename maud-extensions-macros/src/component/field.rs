// Semantic component field classification.
use quote::format_ident;
use syn::{Field, GenericArgument, Ident, PathArguments, Result, Type};

use crate::component::attrs::{DefaultValue, FieldAttrs, parse_field_attrs};

pub(crate) struct ComponentField {
    pub(crate) name: Ident,
    pub(crate) builder_name: Ident,
    pub(crate) ty: Type,
    pub(crate) kind: FieldKind,
}

pub(crate) enum FieldKind {
    Prop(PropField),
    Slot(SlotField),
}

#[allow(dead_code)] // Semantic facets staged for later expansion slices.
pub(crate) struct PropField {
    pub(crate) optional: bool,
    pub(crate) repeated: bool,
    pub(crate) default: Option<DefaultValue>,
    pub(crate) each: Option<Ident>,
}

#[allow(dead_code)] // Semantic facets staged for later expansion slices.
pub(crate) struct SlotField {
    pub(crate) optional: bool,
    pub(crate) repeated: bool,
    pub(crate) default: bool,
    pub(crate) each: Option<Ident>,
}

impl ComponentField {
    pub(crate) fn from_syn(field: Field) -> Result<Self> {
        let attrs = parse_field_attrs(&field.attrs)?;
        let name = field
            .ident
            .clone()
            .expect("named fields only should reach ComponentField::from_syn");
        let builder_name = name.clone();
        let ty = field.ty.clone();
        let optional = option_inner(&field.ty).is_some();
        let repeated = vec_inner(&field.ty).is_some();

        if attrs.each.is_some() && !repeated {
            return Err(syn::Error::new(
                attrs.each.as_ref().expect("checked is_some").span,
                "`#[mx(each = ...)]` only applies to `Vec<T>` fields",
            ));
        }

        let kind = classify_kind(attrs, optional, repeated)?;

        Ok(Self {
            name,
            builder_name,
            ty,
            kind,
        })
    }

    pub(crate) fn is_slot(&self) -> bool {
        matches!(self.kind, FieldKind::Slot(_))
    }

    pub(crate) fn slot(&self) -> Option<&SlotField> {
        let FieldKind::Slot(slot) = &self.kind else {
            return None;
        };
        Some(slot)
    }

    pub(crate) fn state_assoc_ident(&self) -> Ident {
        let mut out = String::new();

        for part in self
            .name
            .to_string()
            .split('_')
            .filter(|part| !part.is_empty())
        {
            let mut chars = part.chars();
            if let Some(first) = chars.next() {
                out.extend(first.to_uppercase());
                out.push_str(chars.as_str());
            }
        }

        format_ident!("{}", out)
    }

    pub(crate) fn set_state_ident(&self) -> Ident {
        let state_assoc = self.state_assoc_ident();
        format_ident!("Set{}", state_assoc)
    }

    pub(crate) fn bon_required_internal_setter_ident(&self) -> Ident {
        format_ident!("__mx_{}_internal", self.name)
    }

    pub(crate) fn bon_optional_some_setter_ident(&self) -> Ident {
        format_ident!("__mx_{}_some_internal", self.name)
    }
}

fn classify_kind(attrs: FieldAttrs, optional: bool, repeated: bool) -> Result<FieldKind> {
    if attrs.slot.is_some() {
        return Ok(FieldKind::Slot(SlotField {
            optional,
            repeated,
            default: attrs.default.is_some(),
            each: attrs.each.map(|each| each.setter),
        }));
    }

    if attrs
        .default
        .as_ref()
        .is_some_and(|default| default.expr.is_none())
        && optional
    {
        return Err(syn::Error::new(
            attrs.default.expect("checked is_some").span,
            "`#[mx(default)]` on an `Option<T>` prop is redundant; optional props are already defaultable by absence",
        ));
    }

    if attrs.default.is_some() && repeated {
        return Ok(FieldKind::Prop(PropField {
            optional,
            repeated,
            default: attrs.default,
            each: attrs.each.map(|each| each.setter),
        }));
    }

    if attrs.default.is_some() || attrs.each.is_some() || optional || repeated {
        return Ok(FieldKind::Prop(PropField {
            optional,
            repeated,
            default: attrs.default,
            each: attrs.each.map(|each| each.setter),
        }));
    }

    Ok(FieldKind::Prop(PropField {
        optional: false,
        repeated: false,
        default: None,
        each: None,
    }))
}

fn option_inner(ty: &Type) -> Option<&Type> {
    type_inner(ty, "Option")
}

fn vec_inner(ty: &Type) -> Option<&Type> {
    type_inner(ty, "Vec")
}

fn type_inner<'a>(ty: &'a Type, outer: &str) -> Option<&'a Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if segment.ident != outer {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    let first = arguments.args.first()?;
    let GenericArgument::Type(inner) = first else {
        return None;
    };
    Some(inner)
}
