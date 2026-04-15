use maud::{Markup, Render, html};
use maud_extensions::{component, css, js, surreal_scope_signals_inline};

js! {
    const count = mx.signal(0);
    const active = mx.computed(() => count.value > 0);

    me(".count").bindText(count);
    me().bindClass("active", active);
    me(".inc").on("click", () => count.value++);
}

struct Counter;

impl Render for Counter {
    fn render(&self) -> Markup {
        component! {
            @js-once
            section class="counter" {
                p { "Count: " span class="count" {} }
                button class="inc" type="button" { "+" }
            }
        }
    }
}

css! {
    me {
        border: 1px solid #ddd;
        padding: 12px;
    }
    me.active {
        border-color: #16a34a;
    }
}

fn main() {
    let _page = html! {
        head { (surreal_scope_signals_inline!()) }
        body { (Counter) }
    };
}
