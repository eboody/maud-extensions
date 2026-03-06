use maud_extensions::component;

fn main() {
    let _ = component! {
        div { "a" }
        span { "b" }
    };
}
