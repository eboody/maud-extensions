use maud::Markup;
use maud_extensions::ComponentBuilder;

#[derive(ComponentBuilder)]
struct BadSlots {
    #[slot(default)]
    body: Markup,
    #[slot(default)]
    footer: Markup,
}

fn main() {}
