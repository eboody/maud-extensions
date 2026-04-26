use maud_extensions::Component;

#[derive(Component)]
struct BadEach {
    #[mx(slot, each = action)]
    action: maud::Markup,
}

fn main() {}
