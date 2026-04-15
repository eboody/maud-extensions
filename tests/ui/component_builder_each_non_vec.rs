use maud::Markup;
use maud_extensions::ComponentBuilder;

#[derive(ComponentBuilder)]
struct BadEach {
    #[builder(each = "body_item")]
    body: Markup,
}

fn main() {}
