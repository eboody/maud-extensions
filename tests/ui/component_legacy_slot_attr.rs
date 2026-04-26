use maud::Markup;
use maud_extensions::{Component, Slot};

#[derive(Component)]
struct LegacySlotAttr {
    #[mx(slot)]
    header: Slot<Markup>,
}

fn main() {}
