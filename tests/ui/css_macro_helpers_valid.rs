use maud_extensions::{css, js};

fn main() {
    let _ = css! {
        media!("(min-width: 48rem)", {
            me { padding: rem!(2); }
        })
        container!("card (min-width: 30rem)", {
            me { gap: px!(12); }
        })
        supports!("(display: grid)", {
            me { width: pct!(100); }
        })
        layer!(components, {
            me { margin: em!(1.5); }
        })
        keyframes!(fade_in, {
            from { opacity: 0; }
            to { opacity: 1; }
        })
    };

    let _ = js! {
        me().style.transitionDuration = "150ms";
    };
}
