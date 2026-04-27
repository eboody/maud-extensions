use maud_extensions::Component;

#[derive(Component)]
struct PropDefaultAttr {
    #[mx(default)]
    disabled: bool,
}

fn main() {}
