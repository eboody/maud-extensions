use maud::{Markup, Render, html};
use maud_extensions::ComponentBuilder;

#[derive(ComponentBuilder)]
struct Banner<'a> {
    r#type: &'a str,
    #[slot(default)]
    body: Markup,
}

impl<'a> Render for Banner<'a> {
    fn render(&self) -> Markup {
        html! {
            section data-type=(self.r#type) {
                (self.body)
            }
        }
    }
}

fn main() {
    let _ = Banner::new()
        .r#type("notice")
        .body(html! { "Body" })
        .build()
        .render();
}
