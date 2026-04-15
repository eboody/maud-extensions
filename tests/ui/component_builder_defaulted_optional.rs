use maud::{Markup, Render, html};
use maud_extensions::ComponentBuilder;

#[derive(ComponentBuilder)]
struct Card {
    #[builder(default)]
    header: Option<Markup>,
    #[slot(default)]
    body: Markup,
}

impl Render for Card {
    fn render(&self) -> Markup {
        html! {
            article {
                @if let Some(header) = &self.header {
                    header { (header) }
                }
                main { (self.body) }
            }
        }
    }
}

fn main() {
    let _ = Card::new()
        .header(html! { h2 { "Heads up" } })
        .body(html! { "Body" })
        .build()
        .render();
}
