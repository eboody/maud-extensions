use maud::Markup;
use maud_extensions::{self as mx, Component, Slot};

#[derive(Component)]
struct InvalidJsMode {
    #[mx(default)]
    body: Slot<Markup>,
}

#[mx::component]
impl InvalidJsMode {
    js!(later, {
        me().class_add("a");
    });

    render! {
        div { (self.body) }
    }
}

fn main() {}
