use maud_extensions::{Component, Slot};

#[derive(Component)]
struct BadNamedSlot {
    header: Slot<Option<String>>,
}

fn main() {}
