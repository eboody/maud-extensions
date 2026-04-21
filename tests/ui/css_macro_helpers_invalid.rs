use maud_extensions::css;

fn main() {
    css! {
        media!("(min-width: 48rem)")
        rem!(1, 2)
        container!["card (min-width: 30rem)", { me { gap: px!(12); } }]
    }
}
