//! `miao-term-config` — configuration and theming.
//!
//! Planned modules:
//! - `schema`  config model (keys aligned with ghostty/alacritty for import)
//! - `import`  ghostty config and alacritty toml/yaml import
//! - `theme`   built-in themes (Nord, Dracula, …) and custom colors

/// Config crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
