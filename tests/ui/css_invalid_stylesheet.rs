use maud_extensions::css;

fn main() {
    css!("body { color: red;");

    let _ = css!(r#".card { display: block; "#);
}
