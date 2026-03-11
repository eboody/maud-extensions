use maud_extensions::{component, css, js};

fn main() {
    js! {
        me().class_add("ready");
    }

    let _ = component! {
        @js-once
        @js-always
        div { "bad" }
    };

    css! {}
}
