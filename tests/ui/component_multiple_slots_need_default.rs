use maud::Markup;
use maud_extensions::{Component, Slot};

#[derive(Component)]
struct AmbiguousSlots {
    header: Slot<Markup>,
    body: Slot<Markup>,
}

fn main() {}
