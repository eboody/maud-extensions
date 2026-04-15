use maud::{Markup, Render, html};
use maud_extensions_runtime::prelude::*;

// Runtime slots are still supported for open caller-owned child structure.
// For new typed shells/layouts, prefer `#[derive(ComponentBuilder)]` in
// `maud-extensions` when the content regions can be expressed as fields.

struct Card;

impl Render for Card {
    fn render(&self) -> Markup {
        html! {
            article class="card" {
                header { (named_slot("header")) }
                main { (slot()) }
            }
        }
    }
}

struct Title<'a> {
    text: &'a str,
}

impl<'a> Render for Title<'a> {
    fn render(&self) -> Markup {
        html! { h2 { (self.text) } }
    }
}

fn main() {
    let _view = html! {
        (Card.with_children(html! {
            (Title { text: "Slots" }.in_slot("header"))
            p { "Body content" }
        }))
    };
}
