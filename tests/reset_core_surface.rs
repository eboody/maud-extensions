use maud::html;
use maud_extensions::{css, js};

#[test]
fn css_and_js_can_be_emitted_inline_inside_html() {
    let markup = html! {
        article class="card" {
            h2 { "Title" }
            p { "Body" }

            (css! {
                me { padding: rem!(1); }
            })

            (js! {
                me().class_add("ready");
            })
        }
    }
    .into_string();

    assert!(markup.contains("<style data-mx-css-id="));
    assert!(markup.contains("padding:1rem"));
    assert!(markup.contains("<script>"));
    assert!(markup.contains("class_add"));
}

#[test]
fn css_and_js_can_define_named_helpers() {
    css!(card_css, {
        me { gap: px!(12); }
    });

    js!(card_js, once, {
        me().class_add("ready");
    });

    let markup = html! {
        article class="card" {
            (card_css())
            (card_js())
        }
    }
    .into_string();

    assert!(markup.contains("gap:12px"));
    assert!(markup.contains("data-mx-js-ran"));
}
