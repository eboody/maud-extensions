use maud::Markup;
use maud_extensions::ComponentBuilder;

#[derive(ComponentBuilder)]
struct InvalidSlotOptional {
    #[slot(optional)]
    header: Option<Markup>,
}

fn main() {}
