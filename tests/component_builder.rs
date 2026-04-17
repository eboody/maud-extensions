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
fn component_builder_still_builds_component_values() {
    let built = Card::new()
        .tone(CardTone::Warning)
        .body(html! { p { "Built card" } })
        .build();

    let rendered = built.render().into_string();
    assert!(rendered.contains("class=\"card warning\""));
    assert!(rendered.contains("<p>Built card</p>"));
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

#[derive(ComponentBuilder)]
struct OptionalCard {
    label: Option<&'static str>,
    #[slot]
    header: Option<Markup>,
    #[slot(default)]
    body: Markup,
}

impl Render for OptionalCard {
    fn render(&self) -> Markup {
        html! {
            article {
                @if let Some(label) = self.label {
                    p class="label" { (label) }
                }
                @if let Some(header) = &self.header {
                    header { (header) }
                }
                main { (self.body) }
            }
        }
    }
}

#[test]
fn component_builder_optional_fields_support_maybe_setters() {
    let rendered = OptionalCard::new()
        .maybe_label(Some("Heads up"))
        .maybe_header(Some(html! { h2 { "Notice" } }))
        .body(html! { "Body" })
        .render()
        .into_string();

    assert!(rendered.contains("<p class=\"label\">Heads up</p>"));
    assert!(rendered.contains("<header><h2>Notice</h2></header>"));

    let rendered = OptionalCard::new()
        .maybe_label(None)
        .maybe_header(None)
        .body(html! { "Body" })
        .render()
        .into_string();

    assert!(!rendered.contains("class=\"label\""));
    assert!(!rendered.contains("<header>"));
}
