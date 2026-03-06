use maud_extensions::component;

fn main() {
    let flag = true;
    let _ = component! {
        @if flag {
            div { "x" }
        }
    };
}
