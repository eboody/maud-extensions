use maud::html;
use maud_extensions::{css, js};

fn status_card(message: &str) -> maud::Markup {
    html! {
        article class="status-card" {
            h2 { "System status" }
            p class="message" { (message) }

            (css! {
                me {
                    border: 1px solid #ddd;
                    padding: 12px;
                }
                me.ready {
                    border-color: #16a34a;
                }
            })

            (js!(once, {
                me().class_add("ready");
            }))
        }
    }
}

fn main() {
    let _markup = status_card("All systems operational");
}
