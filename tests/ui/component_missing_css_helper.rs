use maud_extensions::{component, js};

fn main() {
    js! {
        me().class_add("ready");
    }

    let _ = component! {
        div { "missing css helper" }
    };
}
