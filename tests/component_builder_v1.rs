#![cfg(feature = "components")]

use maud::{Markup, Render, html};
use maud_extensions::{Component, Slot};

#[derive(Component, Debug)]
struct Badge {
    label: String,
    tone: Option<String>,
    count: usize,
}

#[derive(Component, Debug)]
struct Card {
    title: String,
    header: Slot<Markup>,
    #[mx(default)]
    body: Slot<Markup>,
    footer: Slot<Markup>,
    #[mx(each = action)]
    actions: Slot<Vec<Markup>>,
}

impl Badge {
    fn css() -> Markup {
        maud::PreEscaped(String::new())
    }

    fn js() -> Markup {
        maud::PreEscaped(String::new())
    }
}

impl Render for Badge {
    fn render(&self) -> Markup {
        html! {
            span class="badge" {
                (Self::css())
                (Self::js())
                (self.label)
                @if let Some(tone) = &self.tone {
                    " "
                    (tone)
                }
            }
        }
    }
}

impl Card {
    fn css() -> Markup {
        maud_extensions::css! {
            me .actions {
                display: flex;
                gap: 0.5rem;
            }
        }
    }

    fn js() -> Markup {
        maud_extensions::js!(once, {
            me().class_add("ready");
        })
    }
}

impl Render for Card {
    fn render(&self) -> Markup {
        html! {
            article class="card" {
                (Self::css())
                (Self::js())
                header class="header" { (self.header) }
                h2 { (self.title) }
                div class="body" { (self.body) }
                footer class="footer" { (self.footer) }
                div class="actions" { (self.actions) }
            }
        }
    }
}

#[test]
fn component_v1_uses_bon_backed_new_and_build() {
    let badge = Badge::new().label("New").tone("warm").count(0).build();

    assert_eq!(badge.label, "New");
    assert_eq!(badge.tone.as_deref(), Some("warm"));
    assert_eq!(badge.count, 0);
}



#[test]
fn component_v1_supports_optional_props_by_absence() {
    let badge = Badge::new().label("Stable").count(0).build();

    assert_eq!(badge.label, "Stable");
    assert_eq!(badge.tone, None);
    assert_eq!(badge.count, 0);
}

#[test]
fn component_v1_builder_can_render_when_component_implements_render() {
    let markup = Badge::new()
        .label("Live")
        .tone("warm")
        .count(0)
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
fn component_v1_repeated_slots_support_each_style_renderable_setters() {
    let markup = Card::new()
        .title("Profile")
        .header(html! { em { "Heads up" } })
        .child(html! { p { "Body" } })
        .action(html! { button { "Save" } })
        .action(html! { button { "Cancel" } })
        .render()
        .into_string();

    assert!(markup.contains("<header class=\"header\"><em>Heads up</em></header>"));
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
        .header(html! { span { "Welcome" } })
        .footer(html! { button { "Save" } })
        .actions(vec![html! { button { "One" } }, html! { button { "Two" } }])
        .build();

    let markup = card.render().into_string();
    assert!(
        markup.contains("<div class=\"actions\"><button>One</button><button>Two</button></div>")
    );
}
