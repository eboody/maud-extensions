use maud_extensions::{component, css, js};

fn main() {
    js! {}

    let ready = true;
    let _view = component! {
        article class=(if ready { "ready" } else { "waiting" }) {
            p { "valid" }
        }
    };

    css! {}
}
