use maud::html;
use maud_extensions::{css, js};

fn main() {
    css!(card_border, {
        .card { border: 1px solid #ddd; }
    });

    js!(card_js, once, {
        me().class_add("ready");
    });

    let _ = html! {
        div class="card" {
            (card_border())
            (card_js())
            "ok"
        }
    };
}
