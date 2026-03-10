use maud::{Markup, Render, html};
use maud_extensions_runtime::prelude::*;

struct Panel;

impl Render for Panel {
    fn render(&self) -> Markup {
        html! {
            section class="panel" {
                header { (named_slot("header")) }
                main { (slot()) }
            }
        }
    }
}

struct DefaultOnly;

impl Render for DefaultOnly {
    fn render(&self) -> Markup {
        html! {
            div class="default-only" {
                (slot())
            }
        }
    }
}

struct OuterShell;

impl Render for OuterShell {
    fn render(&self) -> Markup {
        html! {
            div class="outer-shell" {
                aside class="outer-header" { (named_slot("header")) }
                section class="outer-body" { (slot()) }
            }
        }
    }
}

struct InnerCard;

impl Render for InnerCard {
    fn render(&self) -> Markup {
        html! {
            article class="inner-card" {
                h2 class="inner-title" { (named_slot("title")) }
                div class="inner-content" { (slot()) }
            }
        }
    }
}

struct HeaderTitle {
    text: &'static str,
}

impl Render for HeaderTitle {
    fn render(&self) -> Markup {
        html! { h1 { (self.text) } }
    }
}

struct CardTitle {
    text: &'static str,
}

impl Render for CardTitle {
    fn render(&self) -> Markup {
        html! { span class="card-title" { (self.text) } }
    }
}

struct SlotOutsideContext;

impl Render for SlotOutsideContext {
    fn render(&self) -> Markup {
        html! {
            div class="no-slots" {
                (slot())
                (named_slot("header"))
            }
        }
    }
}

#[test]
fn slots_route_default_and_named_children() {
    let rendered = html! {
        (Panel.with_children(html! {
            (HeaderTitle { text: "Dashboard" }.in_slot("header"))
            p { "All systems green" }
        }))
    }
    .into_string();

    assert!(rendered.contains("<header><h1>Dashboard</h1></header>"));
    assert!(rendered.contains("<main><p>All systems green</p></main>"));
}

#[test]
fn missing_named_slot_renders_empty() {
    let rendered = html! {
        (Panel.with_children(html! {
            p { "Body only" }
        }))
    }
    .into_string();

    assert!(rendered.contains("<header></header>"));
    assert!(rendered.contains("<main><p>Body only</p></main>"));
}

#[test]
fn unused_named_slot_content_is_ignored() {
    let rendered = html! {
        (DefaultOnly.with_children(html! {
            (HeaderTitle { text: "Ignored" }.in_slot("header"))
            p { "Default body" }
        }))
    }
    .into_string();

    assert!(rendered.contains("<div class=\"default-only\"><p>Default body</p></div>"));
    assert!(!rendered.contains("Ignored"));
}

#[test]
fn nested_components_keep_slot_contexts_isolated() {
    let rendered = html! {
        (OuterShell.with_children(html! {
            (HeaderTitle { text: "Outer Header" }.in_slot("header"))
            (InnerCard.with_children(html! {
                (CardTitle { text: "Inner Title" }.in_slot("title"))
                p { "Inner body" }
            }))
        }))
    }
    .into_string();

    assert!(rendered.contains("<aside class=\"outer-header\"><h1>Outer Header</h1></aside>"));
    assert!(
        rendered.contains(
            "<h2 class=\"inner-title\"><span class=\"card-title\">Inner Title</span></h2>"
        )
    );
    assert!(rendered.contains("<div class=\"inner-content\"><p>Inner body</p></div>"));
}

#[test]
fn slot_functions_outside_with_children_context_are_empty() {
    let rendered = SlotOutsideContext.render().into_string();
    assert!(rendered.contains("<div class=\"no-slots\"></div>"));
}
