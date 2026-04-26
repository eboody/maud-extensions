use maud_extensions::Component;

#[derive(Component)]
struct BadDefaultSlot {
    #[mx(slot, default)]
    body: Option<maud::Markup>,
}

fn main() {}
