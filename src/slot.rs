// Runtime slot wrapper types used by generated component builders.
use core::marker::PhantomData;

use maud::{Markup, PreEscaped, Render};

/// Typed slot storage for component fields.
///
/// The type parameter expresses the semantic child contract, while the runtime
/// value stores already-rendered markup.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Slot<T> {
    markup: String,
    marker: PhantomData<fn() -> T>,
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Self {
            markup: String::new(),
            marker: PhantomData,
        }
    }
}

impl<T> Slot<T> {
    /// Returns true when the slot currently stores no rendered content.
    pub fn is_empty(&self) -> bool {
        self.markup.is_empty()
    }

    /// Converts the slot back into Maud markup.
    pub fn into_markup(self) -> Markup {
        PreEscaped(self.markup)
    }
}

impl<T> Render for Slot<T> {
    fn render(&self) -> Markup {
        PreEscaped(self.markup.clone())
    }
}

impl From<Markup> for Slot<Markup> {
    fn from(value: Markup) -> Self {
        Self {
            markup: value.into_string(),
            marker: PhantomData,
        }
    }
}

impl From<Vec<Markup>> for Slot<Vec<Markup>> {
    fn from(value: Vec<Markup>) -> Self {
        let mut slot = Self::default();
        for item in value {
            slot.push(item);
        }
        slot
    }
}

impl<T> Slot<Vec<T>> {
    /// Appends one rendered item to this repeated slot.
    pub fn push(&mut self, value: Markup) {
        self.markup.push_str(&value.into_string());
    }
}

/// Convenience alias for repeated markup-backed slot storage.
pub type Slots = Slot<Vec<Markup>>;
