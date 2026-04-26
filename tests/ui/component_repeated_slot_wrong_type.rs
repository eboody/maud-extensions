use maud_extensions::Component;

#[derive(Component)]
struct BadRepeatedSlot {
    #[mx(slot, each = action)]
    actions: Vec<String>,
}

fn main() {}
