use maud_extensions::{css, inline_css};

fn main() {
    css!("body { color: red;");

    let _ = inline_css!(r#".card { display: block; "#);
}
