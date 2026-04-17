use maud::{Markup, Render, html};
use maud_extensions::ComponentBuilder;

#[derive(ComponentBuilder)]
struct Banner<'a> {
    label: &'a str,
    subtitle: Option<&'a str>,
    #[slot]
    header: Option<Markup>,
    #[slot(default)]
    body: Markup,
}

impl<'a> Render for Banner<'a> {
    fn render(&self) -> Markup {
        html! {
            section {
                @if let Some(subtitle) = self.subtitle {
                    p class="subtitle" { (subtitle) }
                }
                @if let Some(header) = &self.header {
                    header { (header) }
                }
                p data-label=(self.label) { (self.body) }
            }
        }
    }
}

fn main() {
    let subtitle = Some("subtitle");
    let header = Some(html! { h2 { "Heads up" } });

    let _ = Banner::new()
        .label("notice")
        .maybe_subtitle(subtitle)
        .maybe_header(header)
        .body(html! { "Body" })
        .build()
        .render();
}
