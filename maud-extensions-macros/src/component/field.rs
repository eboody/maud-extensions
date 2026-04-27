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
        let slot_inner = slot_inner(&field.ty);
        let slot_content_ty = slot_inner.unwrap_or(&field.ty);
        let is_slot_field = slot_inner.is_some() || is_maud_extensions_slots(&field.ty);
        let optional = option_inner(slot_content_ty).is_some();
        let repeated = vec_inner(slot_content_ty).is_some() || is_maud_extensions_slots(&field.ty);

        if attrs.each.is_some() && !repeated {
            return Err(syn::Error::new(
                attrs.each.as_ref().expect("checked is_some").span,
                "`#[mx(each = ...)]` only applies to `Vec<T>` fields",
            ));
        }

        let kind = classify_kind(attrs, is_slot_field, optional, repeated)?;

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

    pub(crate) fn slot_inner_ty(&self) -> Option<&Type> {
        slot_inner(&self.ty)
    }

    pub(crate) fn repeated_slot_item_ty(&self) -> Option<&Type> {
        self.slot_inner_ty()
            .and_then(|inner| vec_inner(inner))
            .or_else(|| vec_inner(&self.ty))
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

fn classify_kind(
    attrs: FieldAttrs,
    is_slot_field: bool,
    optional: bool,
    repeated: bool,
) -> Result<FieldKind> {
    if is_slot_field {
        if attrs.default.as_ref().is_some_and(|default| default.expr.is_some()) {
            return Err(syn::Error::new(
                attrs.default.expect("checked is_some").span,
                "`#[mx(default = ...)]` is reserved for slot selection; use bare `#[mx(default)]` on the default slot",
            ));
        }

        return Ok(FieldKind::Slot(SlotField {
            optional,
            repeated,
            default: attrs.default.is_some(),
            each: attrs.each.map(|each| each.setter),
        }));
    }

    if attrs.default.is_some() {
        return Err(syn::Error::new(
            attrs.default.expect("checked is_some").span,
            "`#[mx(default)]` is reserved for selecting the default slot; use Rust `Default` or `Option<T>` for non-slot defaults",
        ));
    }

    if attrs.each.is_some() || optional || repeated {
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

fn slot_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if segment.ident != "Slot" {
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

fn is_maud_extensions_slots(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };

    let segments = type_path.path.segments.iter().collect::<Vec<_>>();
    match segments.as_slice() {
        [single] => single.ident == "Slots",
        [.., penultimate, last] => penultimate.ident == "maud_extensions" && last.ident == "Slots",
        _ => false,
    }
}
