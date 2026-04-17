use maud_extensions::{component, css, js};

fn main() {
    js! {}

    css! { "card_border", {
        .card { border: 1px solid #ddd; }
    } }

    let _ = card_border();

    let _view = component! {
        div class="card" { "ok" }
    };

    css! {}
}
