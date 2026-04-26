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
    #[mx(slot)]
    header: Option<maud::Markup>,
    #[mx(slot, default)]
    body: maud::Markup,
    #[mx(slot, each = action)]
    actions: Vec<maud::Markup>,
    #[mx(slot)]
    footer: Option<maud::Markup>,
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
                @if let Some(header) = &self.header {
                    header class="header" { (header) }
                }
                h2 { (self.title) }
                div class="body" { (self.body) }
                @if !self.actions.is_empty() {
                    div class="actions" {
                        @for action in &self.actions {
                            (action)
                        }
                    }
                }
                @if let Some(footer) = &self.footer {
                    footer class="footer" { (footer) }
                }
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

#[test]
fn component_v1_named_optional_slots_accept_renderables() {
    let markup = Card::new()
        .title("Profile")
        .header(html! { span { "Welcome" } })
        .child(html! { p { "Body" } })
        .footer(html! { button { "Save" } })
        .render()
        .into_string();

    assert!(markup.contains("<header class=\"header\"><span>Welcome</span></header>"));
    assert!(markup.contains("<footer class=\"footer\"><button>Save</button></footer>"));
}

#[test]
fn component_v1_named_optional_slots_support_maybe_setters() {
    let markup = Card::new()
        .title("Profile")
        .maybe_header(Some(html! { em { "Heads up" } }))
        .child(html! { p { "Body" } })
        .maybe_footer(None::<maud::Markup>)
        .render()
        .into_string();

    assert!(markup.contains("<header class=\"header\"><em>Heads up</em></header>"));
    assert!(!markup.contains("class=\"footer\""));
}

#[test]
fn component_v1_repeated_slots_support_each_style_renderable_setters() {
    let markup = Card::new()
        .title("Profile")
        .child(html! { p { "Body" } })
        .action(html! { button { "Save" } })
        .action(html! { button { "Cancel" } })
        .render()
        .into_string();

    assert!(
        markup
            .contains("<div class=\"actions\"><button>Save</button><button>Cancel</button></div>")
    );
}

#[test]
fn component_v1_repeated_slots_keep_bulk_vec_setter() {
    let card = Card::new()
        .title("Profile")
        .body(html! { p { "Body" } })
        .actions(vec![html! { button { "One" } }, html! { button { "Two" } }])
        .build();

    let markup = card.render().into_string();
    assert!(
        markup.contains("<div class=\"actions\"><button>One</button><button>Two</button></div>")
    );
}
