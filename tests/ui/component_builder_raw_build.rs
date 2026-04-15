use maud::{Markup, Render, html};
use maud_extensions::ComponentBuilder;

#[derive(ComponentBuilder)]
struct Card {
    r#build: Markup,
}

impl Render for Card {
    fn render(&self) -> Markup {
        html! { article { (self.r#build) } }
    }
}

fn main() {
    let _ = Card::new().r#build(html! { "Body" }).build().render();
}
