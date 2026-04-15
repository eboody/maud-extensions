use maud::{Markup, Render, html};
use maud_extensions::{component, css, js, surreal_scope_inline};

js! {
    me().class_add("ready");
}

struct StatusCard<'a> {
    message: &'a str,
}

impl<'a> Render for StatusCard<'a> {
    fn render(&self) -> Markup {
        component! {
            @js-once
            article class="status-card" {
                h2 { "System status" }
                p class="message" { (self.message) }
            }
        }
    }
}

css! {
    me {
        border: 1px solid #ddd;
        padding: 12px;
    }
    me.ready {
        border-color: #16a34a;
    }
}

fn main() {
    let _page = html! {
        head { (surreal_scope_inline!()) }
        body { (StatusCard { message: "All systems operational" }) }
    };
}
