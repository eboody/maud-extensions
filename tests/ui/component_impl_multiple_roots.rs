use maud::Markup;
use maud_extensions::{self as mx, Component, Slot};

#[derive(Component)]
struct BadRoots {
    #[mx(default)]
    body: Slot<Markup>,
}

#[mx::component]
impl BadRoots {
    render! {
        div { (self.body) }
        span { "extra" }
    }
}

fn main() {}
