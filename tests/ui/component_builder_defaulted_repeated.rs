use maud::{Markup, Render, html};
use maud_extensions::ComponentBuilder;

#[derive(ComponentBuilder)]
struct List {
    #[builder(default)]
    #[builder(each = "item")]
    items: Vec<Markup>,
}

impl Render for List {
    fn render(&self) -> Markup {
        html! { ul { @for item in &self.items { li { (item) } } } }
    }
}

fn main() {
    let _ = List::new().item(html! { "one" }).build().render();
}
