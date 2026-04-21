use maud::Markup;
use maud_extensions::ComponentBuilder;

#[derive(ComponentBuilder)]
struct InvalidEachName {
    #[builder(each = "not-valid-name")]
    items: Vec<Markup>,
}

fn main() {}
