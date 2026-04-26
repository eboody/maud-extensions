#![cfg(feature = "components")]

use maud::{Markup, Render};
use maud_extensions::{self as mx, Component, ComponentRender, Slot};

#[derive(Component, Debug)]
struct Card {
    title: String,
    #[mx(default)]
    body: Slot<Markup>,
}

#[mx::component]
impl Card {
    css! {
        me {
            padding: 1rem;
        }
    }

    js!(once, {
        me().class_add("ready");
    });

    render! {
        article.card {
            h2 { (self.title) }
            div.body { (self.body) }
        }
    }
}

impl Render for Card {
    fn render(&self) -> Markup {
        ComponentRender::__mx_render(self)
    }
}

#[test]
fn component_impl_macro_generates_hidden_css_js_and_render_hooks() {
    let markup = Card::new()
        .title("Profile")
        .child(mx::css! { me { color: red; } })
        .render()
        .into_string();

    let css = Card::__mx_css().into_string();
    let js = Card::__mx_js().into_string();

    assert!(markup.contains("<article class=\"card\">"));
    assert!(css.contains("padding:1rem"));
    assert!(js.contains("class_add(\"ready\")"));
}

#[test]
fn component_builder_render_auto_injects_impl_block_css_and_js() {
    let markup = Card::new()
        .title("Profile")
        .child(mx::css! { me { color: red; } })
        .render()
        .into_string();

    assert!(markup.contains("<style"));
    assert!(markup.contains("padding:1rem"));
    assert!(markup.contains("<script>"));
    assert!(markup.contains("class_add(\"ready\")"));
    assert!(markup.contains("<article class=\"card\">"));
}
