use maud_extensions::{component, css};

fn main() {
    css! {
        me { color: #111; }
    }

    let _ = component! {
        div { "missing js helper" }
    };
}
