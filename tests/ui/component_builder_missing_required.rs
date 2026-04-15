use maud::{Markup, Render, html};
use maud_extensions::ComponentBuilder;

#[derive(ComponentBuilder)]
struct Card {
    title: &'static str,
    #[slot(default)]
    body: Markup,
}

impl Render for Card {
    fn render(&self) -> Markup {
        html! { article { h2 { (self.title) } (self.body) } }
    }
}

fn main() {
    let _ = Card::new().title("Only title").build();
}
