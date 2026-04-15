use maud::{Markup, Render, html};
use maud_extensions::ComponentBuilder;

#[derive(Clone, Copy)]
enum CardTone {
    Info,
    Warning,
}

impl CardTone {
    fn class_name(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
        }
    }
}

#[derive(ComponentBuilder)]
struct Card {
    tone: CardTone,
    #[builder(default)]
    elevated: bool,
    #[slot]
    header: Option<Markup>,
    #[slot(default)]
    body: Markup,
    #[slot]
    #[builder(each = "action")]
    actions: Vec<ActionButton>,
}

impl Render for Card {
    fn render(&self) -> Markup {
        html! {
            article class={ "card " (self.tone.class_name()) @if self.elevated { " elevated" } } {
                @if let Some(header) = &self.header {
                    header class="card-header" { (header) }
                }
                main class="card-body" { (self.body) }
                @if !self.actions.is_empty() {
                    footer class="card-actions" {
                        @for action in &self.actions {
                            (action)
                        }
                    }
                }
            }
        }
    }
}

struct Heading {
    text: &'static str,
}

impl Render for Heading {
    fn render(&self) -> Markup {
        html! { h2 { (self.text) } }
    }
}

#[derive(Clone)]
struct ActionButton {
    label: &'static str,
}

impl Render for ActionButton {
    fn render(&self) -> Markup {
        html! { button type="button" { (self.label) } }
    }
}

#[test]
fn component_builder_builds_required_optional_and_repeated_fields() {
    let rendered = Card::new()
        .tone(CardTone::Info)
        .elevated(true)
        .header(Heading { text: "Status" })
        .body(html! { p { "All systems green" } })
        .action(ActionButton { label: "Retry" })
        .action(ActionButton { label: "Dismiss" })
        .build()
        .render()
        .into_string();

    assert!(rendered.contains("class=\"card info elevated\""));
    assert!(rendered.contains("<header class=\"card-header\"><h2>Status</h2></header>"));
    assert!(rendered.contains("<main class=\"card-body\"><p>All systems green</p></main>"));
    assert!(
        rendered.contains(
            "<footer class=\"card-actions\"><button type=\"button\">Retry</button><button type=\"button\">Dismiss</button></footer>"
        )
    );
}

#[test]
fn component_builder_builder_alias_and_bulk_vec_setter_work() {
    let built: Card = Card::builder()
        .tone(CardTone::Warning)
        .body(html! { p { "Watch it" } })
        .actions(vec![ActionButton {
            label: "Acknowledge",
        }])
        .into();

    let rendered = built.render().into_string();
    assert!(rendered.contains("class=\"card warning\""));
    assert!(rendered.contains("<button type=\"button\">Acknowledge</button>"));
    assert!(!rendered.contains("card-header"));
}
