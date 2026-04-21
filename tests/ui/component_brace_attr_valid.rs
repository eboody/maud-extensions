use maud_extensions::{component, css, js};

fn main() {
    js! {}

    let elevated = true;
    let _view = component! {
        article class={ "card" @if elevated { " elevated" } } {
            p { "valid" }
        }
    };

    css! {}
}
