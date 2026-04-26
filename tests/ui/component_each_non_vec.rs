use maud::Markup;
use maud_extensions::{Component, Slot};

#[derive(Component)]
struct BadEach {
    #[mx(each = action)]
    action: Slot<Markup>,
}

fn main() {}
