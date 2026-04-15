use maud_extensions::{font_face, font_faces};

fn main() {
    let _single = maud::html! {
        style {
            (font_face!(
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/examples/assets/demo-font.woff2"
                ),
                "Demo Sans"
            ))
        }
    };

    let _multiple = maud::html! {
        style {
            (font_faces!(
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/examples/assets/demo-font.woff2"
                ), "Demo Sans";
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/examples/assets/demo-font-bold.woff2"
                ), "Demo Sans", "700", "normal"
            ))
        }
    };
}
