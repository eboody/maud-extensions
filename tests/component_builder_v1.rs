#![cfg(feature = "components")]

use maud::{Markup, Render, html};
use maud_extensions::Component;

#[derive(Component, Debug)]
struct Badge {
    label: String,
    tone: Option<String>,
    #[mx(default = 0)]
    count: usize,
}

#[derive(Component, Debug)]
struct Card {
    title: String,
    #[mx(slot, default)]
    body: maud::Markup,
}

impl Render for Badge {
    fn render(&self) -> Markup {
        html! {
            span class="badge" {
                (self.label)
                @if let Some(tone) = &self.tone {
                    " "
                    (tone)
                }
            }
        }
    }
}

impl Render for Card {
    fn render(&self) -> Markup {
        html! {
            article class="card" {
                h2 { (self.title) }
                div class="body" { (self.body) }
            }
        }
    }
}

#[test]
fn component_v1_uses_bon_backed_new_and_build() {
    let badge = Badge::new().label("New").tone("warm").build();

    assert_eq!(badge.label, "New");
    assert_eq!(badge.tone.as_deref(), Some("warm"));
    assert_eq!(badge.count, 0);
}

#[test]
fn component_v1_supports_optional_props_by_absence() {
    let badge = Badge::new().label("Stable").build();

    assert_eq!(badge.label, "Stable");
    assert_eq!(badge.tone, None);
    assert_eq!(badge.count, 0);
}

#[test]
fn component_v1_builder_can_render_when_component_implements_render() {
    let markup = Badge::new()
        .label("Live")
        .tone("warm")
        .render()
        .into_string();

    assert!(markup.contains("<span class=\"badge\">Live warm</span>"));
}

#[test]
fn component_v1_default_slot_supports_child_alias() {
    let markup = Card::new()
        .title("Settings")
        .child(html! { p { "Profile details" } })
        .render()
        .into_string();

    assert!(markup.contains("<h2>Settings</h2>"));
    assert!(markup.contains("<div class=\"body\"><p>Profile details</p></div>"));
}

#[test]
fn component_v1_default_slot_supports_named_slot_setter_accepting_render() {
    let card = Card::new()
        .title("Account")
        .body(html! { strong { "Details" } })
        .build();

    let markup = card.render().into_string();
    assert!(markup.contains("<div class=\"body\"><strong>Details</strong></div>"));
}
