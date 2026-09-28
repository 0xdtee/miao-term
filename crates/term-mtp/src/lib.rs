//! `miao-term-mtp` — the MTP control plane (Miaotty Terminal Protocol).
//!
//! Planned modules:
//! - `wire`      NDJSON envelope (reuse the existing `mtp` types)
//! - `transport` Unix socket / Windows named pipe (`\\.\pipe\miaotty`)
//! - `server`    Rust MTP host; dispatch (core/agent/history/pane)
//! - `registry`  agent-state + command-history registries
//! - `events`    subscription push (agent.state / cwd.changed / history.changed)

/// MTP crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
