//! `miao-term-widget` — windowing and platform integration.
//!
//! Planned modules:
//! - `window`    winit event loop + wgpu surface + frame pacing
//! - `input`     keyboard/mouse/focus/drag-drop → `core::input`; IME handling
//! - `clipboard` platform clipboard
//! - `compose`   `TerminalView` (custom-drawn) and egui (panels/chrome) share one surface
//! - `host`      [`Host`] callbacks (open URL, notify, clipboard, title, cwd)

/// Widget crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
