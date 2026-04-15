use maud_extensions::ComponentBuilder;

mod foreign {
    #[derive(Clone)]
    pub struct Markup;
}

#[derive(ComponentBuilder)]
struct Wrapper {
    body: foreign::Markup,
}

fn main() {
    let _ = Wrapper::new().body(foreign::Markup).build();
}
