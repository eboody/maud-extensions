use maud_extensions::{Component, Slot};

#[derive(Component)]
struct BadRepeatedSlot {
    #[mx(each = action)]
    actions: Slot<Vec<String>>,
}

fn main() {}
