use std::{cell::RefCell, collections::HashMap, fmt::Write as _};

use maud::{Markup, PreEscaped, Render, html};

const SLOT_START_PREFIX: &str = "<!--mx-slot-start:";
const SLOT_START_SUFFIX: &str = "-->";
const SLOT_END_MARKER: &str = "<!--mx-slot-end-->";

#[derive(Default)]
struct SlotPayload {
    default_html: String,
    named_html: HashMap<String, String>,
}

thread_local! {
    static SLOT_STACK: RefCell<Vec<SlotPayload>> = RefCell::new(Vec::new());
}

pub struct Slotted<T: Render> {
    value: T,
    slot_name: String,
}

impl<T: Render> Slotted<T> {
    pub fn new(value: T, slot_name: String) -> Self {
        Self { value, slot_name }
    }
}

impl<T: Render> Render for Slotted<T> {
    fn render(&self) -> Markup {
        let mut start_marker =
            String::with_capacity(SLOT_START_PREFIX.len() + self.slot_name.len() * 2 + 3);
        start_marker.push_str(SLOT_START_PREFIX);
        start_marker.push_str(&encode_slot_name(&self.slot_name));
        start_marker.push_str(SLOT_START_SUFFIX);

        html! {
            (PreEscaped(start_marker))
            (self.value.render())
            (PreEscaped(SLOT_END_MARKER.to_string()))
        }
    }
}

pub trait InSlotExt: Render + Sized {
    fn in_slot(self, slot_name: &str) -> Slotted<Self> {
        Slotted::new(self, slot_name.to_string())
    }
}

impl<T> InSlotExt for T where T: Render {}

pub struct SlottedComponent<T: Render> {
    component: T,
    children_html: String,
}

impl<T: Render> SlottedComponent<T> {
    pub fn new(component: T, children: Markup) -> Self {
        Self {
            component,
            children_html: children.into_string(),
        }
    }
}

impl<T: Render> Render for SlottedComponent<T> {
    fn render(&self) -> Markup {
        let payload = collect_slots_from_children(self.children_html.clone());
        SLOT_STACK.with(|stack| {
            stack.borrow_mut().push(payload);
        });

        struct SlotGuard;
        impl Drop for SlotGuard {
            fn drop(&mut self) {
                SLOT_STACK.with(|stack| {
                    stack.borrow_mut().pop();
                });
            }
        }

        let _guard = SlotGuard;
        self.component.render()
    }
}

pub trait WithChildrenExt: Render + Sized {
    fn with_children(self, children: Markup) -> SlottedComponent<Self> {
        SlottedComponent::new(self, children)
    }
}

impl<T> WithChildrenExt for T where T: Render {}

pub mod prelude {
    pub use crate::{InSlotExt, WithChildrenExt, named_slot, slot};
}

pub fn slot() -> Markup {
    current_slot_html(|payload| payload.default_html.clone())
        .map(PreEscaped)
        .unwrap_or_else(empty_markup)
}

pub fn named_slot(slot_name: &str) -> Markup {
    current_slot_html(|payload| payload.named_html.get(slot_name).cloned())
        .flatten()
        .map(PreEscaped)
        .unwrap_or_else(empty_markup)
}

fn current_slot_html<T>(f: impl FnOnce(&SlotPayload) -> T) -> Option<T> {
    SLOT_STACK.with(|stack| {
        let stack = stack.borrow();
        stack.last().map(f)
    })
}

fn empty_markup() -> Markup {
    PreEscaped(String::new())
}

fn collect_slots_from_children(children_html: String) -> SlotPayload {
    let mut payload = SlotPayload::default();
    let mut cursor = 0usize;

    while let Some(start_rel) = children_html[cursor..].find(SLOT_START_PREFIX) {
        let slot_marker_start = cursor + start_rel;
        payload
            .default_html
            .push_str(&children_html[cursor..slot_marker_start]);

        let encoded_name_start = slot_marker_start + SLOT_START_PREFIX.len();
        let Some(name_end_rel) = children_html[encoded_name_start..].find(SLOT_START_SUFFIX) else {
            payload
                .default_html
                .push_str(&children_html[slot_marker_start..]);
            return payload;
        };
        let encoded_name_end = encoded_name_start + name_end_rel;
        let encoded_name = &children_html[encoded_name_start..encoded_name_end];
        let slot_name = decode_slot_name(encoded_name).unwrap_or_else(|| encoded_name.to_string());

        let slot_content_start = encoded_name_end + SLOT_START_SUFFIX.len();
        let Some(slot_end_rel) = children_html[slot_content_start..].find(SLOT_END_MARKER) else {
            payload
                .default_html
                .push_str(&children_html[slot_marker_start..]);
            return payload;
        };
        let slot_content_end = slot_content_start + slot_end_rel;
        let slot_content = &children_html[slot_content_start..slot_content_end];

        payload
            .named_html
            .entry(slot_name)
            .or_default()
            .push_str(slot_content);

        cursor = slot_content_end + SLOT_END_MARKER.len();
    }

    payload.default_html.push_str(&children_html[cursor..]);
    payload
}

fn encode_slot_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len() * 2);
    for byte in name.as_bytes() {
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn decode_slot_name(encoded_name: &str) -> Option<String> {
    if encoded_name.is_empty() || encoded_name.len() % 2 != 0 {
        return None;
    }

    let mut bytes = Vec::with_capacity(encoded_name.len() / 2);
    for chunk in encoded_name.as_bytes().chunks_exact(2) {
        let chunk = std::str::from_utf8(chunk).ok()?;
        let byte = u8::from_str_radix(chunk, 16).ok()?;
        bytes.push(byte);
    }

    String::from_utf8(bytes).ok()
}
