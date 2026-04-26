use maud::Markup;
use maud_extensions::{Component, Slot};

#[derive(Component)]
struct BadDefaultSlot {
    #[mx(default)]
    body: Slot<Option<Markup>>,
}

fn main() {}
