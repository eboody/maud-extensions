use maud::Markup;
use maud_extensions::{self as mx, Component, Slot};

#[derive(Component)]
struct DuplicateJs {
    #[mx(default)]
    body: Slot<Markup>,
}

#[mx::component]
impl DuplicateJs {
    js! {
        me().class_add("a");
    }

    js!(once, {
        me().class_add("b");
    });

    render! {
        div { (self.body) }
    }
}

fn main() {}
