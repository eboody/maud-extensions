use maud_extensions::{component, css, js};

fn main() {
    js! {
        me().class_add("ready");
    }

    let _once = component! {
        @js-once
        article class="card" {
            p { "once" }
        }
    };

    let _always = component! {
        @js-always
        article class="card" {
            p { "always" }
        }
    };

    css! {
        me { border: 1px solid #ddd; }
    }
}
