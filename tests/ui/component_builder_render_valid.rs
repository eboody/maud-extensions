use maud::{Markup, Render, html};
use maud_extensions::ComponentBuilder;

#[derive(ComponentBuilder)]
struct Card<'a> {
    title: &'a str,
    #[slot(default)]
    body: Markup,
}

impl<'a> Render for Card<'a> {
    fn render(&self) -> Markup {
        html! {
            article data-title=(self.title) { (self.body) }
        }
    }
}

fn main() {
    let _ = Card::new()
        .title("Status")
        .body(html! { "Body" })
        .render();
}
