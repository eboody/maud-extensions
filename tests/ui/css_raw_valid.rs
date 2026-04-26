use maud_extensions::css;

fn main() {
    let _ = css! {
        raw!(r#":root { --font-display: 'Newsreader', Georgia, serif; }"#)
    };

    let _ = css! {
        raw!(r#"[data-theme='light'] { color-scheme: light; }"#)
    };
}
