use maud::Markup;
use maud_extensions::{self as mx, Component, Slot};

#[derive(Component)]
struct DuplicateCss {
    #[mx(default)]
    body: Slot<Markup>,
}

#[mx::component]
impl DuplicateCss {
    css! {
        me { color: red; }
    }

    css! {
        me { color: blue; }
    }

    render! {
        div { (self.body) }
    }
}

fn main() {}
