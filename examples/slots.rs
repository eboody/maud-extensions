use maud::{Markup, Render, html};
use maud_extensions_runtime::prelude::*;

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
