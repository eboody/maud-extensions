use maud::{Markup, Render, html};
use maud_extensions_runtime::prelude::*;

struct Layout;

impl Render for Layout {
    fn render(&self) -> Markup {
        html! {
            div class="layout" {
                header { (named_slot("header")) }
                main { (slot()) }
            }
        }
    }
}

struct Title;

impl Render for Title {
    fn render(&self) -> Markup {
        html! { h1 { "Hi" } }
    }
}

fn main() {
    let _ = html! {
        (Layout.with_children(html! {
            (Title.in_slot("header"))
            p { "Body" }
        }))
    };
}
