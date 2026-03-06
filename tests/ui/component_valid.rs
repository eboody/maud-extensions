use maud_extensions::{component, css, js};

fn main() {
    js! {
        me().class_add("ready");
    }

    let _view = component! {
        article class="card" {
            p { "valid" }
        }
    };

    css! {
        me { border: 1px solid #ddd; }
    }
}
