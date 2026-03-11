use maud_extensions::{component, css, js};

fn main() {
    js! {
        me().class_add("ready");
    }

    let _ = component! {
        div { "bad" }
        @js-once
    };

    css! {}
}
