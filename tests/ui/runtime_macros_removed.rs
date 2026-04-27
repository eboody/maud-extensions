use maud_extensions::{signals_inline, surreal_scope_inline, surreal_scope_signals_inline};

fn main() {
    let _ = surreal_scope_inline!();
    let _ = signals_inline!();
    let _ = surreal_scope_signals_inline!();
}
