// Parsed #[mx(...)] metadata attached to component fields.
use proc_macro2::Span;
use syn::{
    Attribute, Error, Expr, Ident, Result, Token,
    parse::{Parse, ParseStream},
};

#[derive(Default)]
pub(crate) struct FieldAttrs {
    pub(crate) default: Option<DefaultValue>,
    pub(crate) each: Option<Each>,
}

pub(crate) struct DefaultValue {
    pub(crate) span: Span,
    pub(crate) expr: Option<Expr>,
}

pub(crate) struct Each {
    pub(crate) span: Span,
    pub(crate) setter: Ident,
}

enum AttrItem {
    Default { span: Span, expr: Option<Expr> },
    Each { span: Span, setter: Ident },
}

impl Parse for AttrItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        let span = ident.span();

        match ident.to_string().as_str() {
            "slot" => Err(Error::new(
                span,
                "slot fields are now declared with `Slot<T>` types; remove `#[mx(slot)]` and change the field type instead",
            )),
            "default" => {
                if input.peek(Token![=]) {
                    input.parse::<Token![=]>()?;
                    Ok(Self::Default {
                        span,
                        expr: Some(input.parse()?),
                    })
                } else {
                    Ok(Self::Default { span, expr: None })
                }
            }
            "each" => {
                input.parse::<Token![=]>()?;
                Ok(Self::Each {
                    span,
                    setter: input.parse()?,
                })
            }
            _ => Err(Error::new(
                ident.span(),
                "unknown mx field attribute; supported forms are `default`, `default = ...`, and `each = name`",
            )),
        }
    }
}

pub(crate) fn parse_field_attrs(attrs: &[Attribute]) -> Result<FieldAttrs> {
    let mut out = FieldAttrs::default();

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("mx")) {
        attr.parse_args_with(|input: ParseStream| {
            while !input.is_empty() {
                let item: AttrItem = input.parse()?;
                apply_item(&mut out, item)?;

                if input.is_empty() {
                    break;
                }

                input.parse::<Token![,]>()?;
            }

            Ok(())
        })?;
    }

    Ok(out)
}

fn apply_item(attrs: &mut FieldAttrs, item: AttrItem) -> Result<()> {
    match item {
        AttrItem::Default { span, expr } => {
            if attrs.default.is_some() {
                return Err(Error::new(span, "duplicate `default` attribute"));
            }
            attrs.default = Some(DefaultValue { span, expr });
        }
        AttrItem::Each { span, setter } => {
            if attrs.each.is_some() {
                return Err(Error::new(span, "duplicate `each = ...` attribute"));
            }
            if setter == "render" || setter == "build" || setter == "new" {
                return Err(Error::new(
                    setter.span(),
                    "reserved builder method name in `each = ...` attribute",
                ));
            }
            attrs.each = Some(Each { span, setter });
        }
    }

    Ok(())
}
