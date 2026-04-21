#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
//! Runtime slot helpers for `maud-extensions`.
//!
//! This crate owns the lower-level string-based slot transport used by
//! `.with_children(...)`, `.in_slot(...)`, `slot()`, and `named_slot()`.
//!
//! Prefer `#[derive(ComponentBuilder)]` from `maud-extensions` for new shell
//! and layout components when the content regions can be expressed as typed
//! fields. Use this crate when you genuinely need open caller-owned child
//! markup, or when you are keeping an existing slot-based component.
//!
//! Support policy:
//! - MSRV: Rust 1.85
//! - Supported Maud version: 0.27
//!
//! The runtime keeps slot parsing conservative. Malformed transport markers fail closed and the
//! original HTML is preserved as default slot content instead of being partially consumed.

use std::{
    cell::RefCell,
    collections::hash_map::RandomState,
    collections::HashMap,
    fmt::Write as _,
    hash::{BuildHasher, Hash, Hasher},
    sync::atomic::{AtomicU64, Ordering},
    sync::OnceLock,
};

use maud::{Markup, PreEscaped, Render, html};

const SLOT_START_PREFIX: &str = "<!--maud-extensions-slot-start:v1:";
const SLOT_END_PREFIX: &str = "<!--maud-extensions-slot-end:v1:";
const SLOT_MARKER_SEPARATOR: char = ':';
const SLOT_MARKER_SUFFIX: &str = "-->";

#[derive(Default)]
struct SlotPayload {
    default_html: String,
    named_html: HashMap<String, String>,
}

thread_local! {
    static SLOT_STACK: RefCell<Vec<SlotPayload>> = const { RefCell::new(Vec::new()) };
}

static SLOT_MARKER_COUNTER: AtomicU64 = AtomicU64::new(1);

fn slot_marker_hasher() -> &'static RandomState {
    static SLOT_MARKER_HASHER: OnceLock<RandomState> = OnceLock::new();
    SLOT_MARKER_HASHER.get_or_init(RandomState::new)
}

fn slot_marker_auth(marker_id: &str, slot_name: &str) -> String {
    let mut hasher = slot_marker_hasher().build_hasher();
    marker_id.hash(&mut hasher);
    slot_name.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// A renderable wrapper that assigns content to a named slot.
///
/// Most code should reach this type through [`InSlotExt::in_slot`] instead of constructing it
/// directly.
pub struct Slotted<T: Render> {
    value: T,
    slot_name: String,
}

impl<T: Render> Slotted<T> {
    /// Creates a slotted wrapper around a renderable value.
    #[must_use]
    pub fn new(value: T, slot_name: String) -> Self {
        Self { value, slot_name }
    }
}

impl<T: Render> Render for Slotted<T> {
    fn render(&self) -> Markup {
        let marker_id = next_slot_marker_id();
        let marker_auth = slot_marker_auth(&marker_id, &self.slot_name);
        let mut start_marker = String::with_capacity(
            SLOT_START_PREFIX.len() + marker_id.len() + self.slot_name.len() * 2 + marker_auth.len() + 16,
        );
        start_marker.push_str(SLOT_START_PREFIX);
        start_marker.push_str(&marker_id);
        start_marker.push(SLOT_MARKER_SEPARATOR);
        start_marker.push_str(&encode_slot_name(&self.slot_name));
        start_marker.push(SLOT_MARKER_SEPARATOR);
        start_marker.push_str(&marker_auth);
        start_marker.push_str(SLOT_MARKER_SUFFIX);

        let mut end_marker = String::with_capacity(
            SLOT_END_PREFIX.len() + marker_id.len() + SLOT_MARKER_SUFFIX.len(),
        );
        end_marker.push_str(SLOT_END_PREFIX);
        end_marker.push_str(&marker_id);
        end_marker.push_str(SLOT_MARKER_SUFFIX);

        html! {
            (PreEscaped(start_marker))
            (self.value.render())
            (PreEscaped(end_marker))
        }
    }
}

/// Extension trait for assigning children to a named slot.
pub trait InSlotExt: Render + Sized {
    /// Tags the rendered value for `named_slot(slot_name)`.
    ///
    /// Slot names are opaque strings. Duplicate names are concatenated in render order.
    #[must_use]
    fn in_slot(self, slot_name: &str) -> Slotted<Self> {
        Slotted::new(self, slot_name.to_string())
    }
}

impl<T> InSlotExt for T where T: Render {}

/// A renderable wrapper that attaches child markup to a component before rendering it.
///
/// This is the transport hook that feeds `.with_children(...)` into the runtime
/// slot stack read by [`slot`] and [`named_slot`].
///
/// Most code should reach this type through [`WithChildrenExt::with_children`] instead of
/// constructing it directly.
pub struct SlottedComponent<T: Render> {
    component: T,
    children_html: String,
}

impl<T: Render> SlottedComponent<T> {
    /// Creates a slotted component wrapper around a component and its children.
    #[must_use]
    pub fn new(component: T, children: Markup) -> Self {
        Self {
            component,
            children_html: children.into_string(),
        }
    }
}

impl<T: Render> Render for SlottedComponent<T> {
    fn render(&self) -> Markup {
        let payload = collect_slots_from_children(&self.children_html);
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

/// Extension trait for attaching slot-aware child markup to a renderable component.
///
/// This is the lower-level transport surface for runtime slots. Prefer
/// `ComponentBuilder` from `maud-extensions` for new typed shell/layout
/// components when the regions can be expressed as fields.
pub trait WithChildrenExt: Render + Sized {
    /// Renders `children` into the slot transport expected by [`slot`] and [`named_slot`].
    #[must_use]
    fn with_children(self, children: Markup) -> SlottedComponent<Self> {
        SlottedComponent::new(self, children)
    }
}

impl<T> WithChildrenExt for T where T: Render {}

/// Common imports for runtime slot-based component composition.
pub mod prelude {
    pub use crate::{InSlotExt, WithChildrenExt, named_slot, slot};
}

/// Renders the default slot for the current slotted component context.
///
/// Outside `.with_children(...)`, this returns empty markup.
#[must_use]
pub fn slot() -> Markup {
    current_slot_html(|payload| payload.default_html.clone())
        .map(PreEscaped)
        .unwrap_or_else(empty_markup)
}

/// Renders a named slot for the current slotted component context.
///
/// Duplicate slot names are concatenated in render order. Outside `.with_children(...)`, this
/// returns empty markup.
#[must_use]
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

fn next_slot_marker_id() -> String {
    format!(
        "{:016x}",
        SLOT_MARKER_COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

fn collect_slots_from_children(children_html: &str) -> SlotPayload {
    let mut payload = SlotPayload::default();
    let mut cursor = 0usize;

    while let Some(start_rel) = children_html[cursor..].find(SLOT_START_PREFIX) {
        let slot_marker_start = cursor + start_rel;
        payload
            .default_html
            .push_str(&children_html[cursor..slot_marker_start]);

        let marker_content_start = slot_marker_start + SLOT_START_PREFIX.len();
        let Some(marker_end_rel) = children_html[marker_content_start..].find(SLOT_MARKER_SUFFIX)
        else {
            payload
                .default_html
                .push_str(&children_html[slot_marker_start..]);
            return payload;
        };
        let marker_content_end = marker_content_start + marker_end_rel;
        let marker_content = &children_html[marker_content_start..marker_content_end];
        let mut marker_parts = marker_content.split(SLOT_MARKER_SEPARATOR);
        let Some(marker_id) = marker_parts.next()
        else {
            payload
                .default_html
                .push_str(&children_html[slot_marker_start..]);
            return payload;
        };
        let Some(encoded_name) = marker_parts.next()
        else {
            payload
                .default_html
                .push_str(&children_html[slot_marker_start..]);
            return payload;
        };
        let Some(marker_auth) = marker_parts.next()
        else {
            payload
                .default_html
                .push_str(&children_html[slot_marker_start..]);
            return payload;
        };
        if marker_id.is_empty() || marker_parts.next().is_some() {
            payload
                .default_html
                .push_str(&children_html[slot_marker_start..]);
            return payload;
        }

        let Some(slot_name) = decode_slot_name(encoded_name) else {
            payload
                .default_html
                .push_str(&children_html[slot_marker_start..]);
            return payload;
        };
        if marker_auth != slot_marker_auth(marker_id, &slot_name) {
            payload
                .default_html
                .push_str(&children_html[slot_marker_start..]);
            return payload;
        }
        let slot_content_start = marker_content_end + SLOT_MARKER_SUFFIX.len();

        let mut end_marker = String::with_capacity(
            SLOT_END_PREFIX.len() + marker_id.len() + SLOT_MARKER_SUFFIX.len(),
        );
        end_marker.push_str(SLOT_END_PREFIX);
        end_marker.push_str(marker_id);
        end_marker.push_str(SLOT_MARKER_SUFFIX);

        let Some(slot_end_rel) = children_html[slot_content_start..].find(&end_marker) else {
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

        cursor = slot_content_end + end_marker.len();
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
    if encoded_name.is_empty() {
        return Some(String::new());
    }

    if encoded_name.len() % 2 != 0 {
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
