use maud_extensions::ComponentBuilder;

#[derive(ComponentBuilder)]
struct Card {
    body: maud::Markup,
}

fn main() {
    let _ = Card::new().body(maud::html! { p { "Live" } }).build();
}
