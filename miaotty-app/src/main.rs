//! miaotty application entry point (scaffold).
//!
//! Will compose the `miao-term` engine (terminal view, windows/tabs/splits,
//! panels, MTP host). For now it only proves the workspace wires together.

fn main() {
    println!(
        "miaotty {} (engine core {})",
        env!("CARGO_PKG_VERSION"),
        miao_term_core::VERSION
    );
}
