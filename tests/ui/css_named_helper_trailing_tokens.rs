use maud_extensions::css;

fn main() {
    css!(card_styles, {
        .card { color: red; }
    }, trailing);
}
