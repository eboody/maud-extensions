use maud::{Markup, Render, html};
use maud_extensions::ComponentBuilder;

#[derive(ComponentBuilder)]
struct Banner {
    header: Option<Markup>,
    maybe_header: &'static str,
    #[slot(default)]
    body: Markup,
}

impl Render for Banner {
    fn render(&self) -> Markup {
        html! {
            section {
                @if let Some(header) = &self.header {
                    header { (header) }
                }
                p data-extra=(self.maybe_header) { (self.body) }
            }
        }
    }
}

fn main() {}
