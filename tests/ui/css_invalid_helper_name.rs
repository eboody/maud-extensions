use maud_extensions::css;

fn main() {
    css! { "not-valid-name", {
        .card { color: red; }
    } }
}
