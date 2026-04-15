use maud::{Markup, Render, html};
use maud_extensions::ComponentBuilder;

#[derive(ComponentBuilder)]
struct Banner<'a> {
    label: &'a str,
    #[slot]
    header: Option<Markup>,
    #[slot(default)]
    body: Markup,
}

impl<'a> Render for Banner<'a> {
    fn render(&self) -> Markup {
        html! {
            section {
                @if let Some(header) = &self.header {
                    header { (header) }
                }
                p data-label=(self.label) { (self.body) }
            }
        }
    }
}

struct Heading;

impl Render for Heading {
    fn render(&self) -> Markup {
        html! { h1 { "Heads up" } }
    }
}

fn main() {
    let _ = Banner::new()
        .label("notice")
        .header(Heading)
        .body(html! { "Body" })
        .build()
        .render();
}
