//! `miao-term-render` — wgpu glyph-grid renderer.
//!
//! Planned modules:
//! - `font`     cosmic-text + swash shaping, R8 glyph atlas
//! - `atlas`    glyph cache (keyed by glyph/style/px), LRU
//! - `grid`     cell → quad instances (bg / underline / strike / cursor / selection)
//! - `pipeline` bg pass + glyph pass + cursor/decoration
//! - `damage`   incremental rebuild of dirty lines

/// Renderer crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
