use maud_extensions::Component;

#[derive(Component)]
struct BadNamedSlot {
    #[mx(slot)]
    header: Option<String>,
}

fn main() {}
