use maud::{Markup, Render, html};
use maud_extensions::ComponentBuilder;

#[derive(ComponentBuilder)]
struct Card {
    r#render: Markup,
}

impl Render for Card {
    fn render(&self) -> Markup {
        html! { article { (self.r#render) } }
    }
}

fn main() {
    let _ = Card::new().r#render(html! { "Body" }).build().render();
}
