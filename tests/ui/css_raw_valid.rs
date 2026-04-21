use maud_extensions::{css, inline_css};

fn main() {
    css! {
        raw!(r#":root { --font-display: 'Newsreader', Georgia, serif; }"#)
    }

    let _ = css();

    let _ = inline_css! {
        raw!(r#"[data-theme='light'] { color-scheme: light; }"#)
    };
}
