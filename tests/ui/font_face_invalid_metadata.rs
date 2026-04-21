use maud_extensions::font_face;

fn main() {
    let _ = font_face!(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/demo-font.woff2"),
        "Fixture Sans",
        "normal; color:red",
        "italic"
    );
}
