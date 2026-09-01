//! Native theme core for Doubao and DoubaoWork, with a DoubaoWork model bridge.
//!
//! Reskins the selected Chromium-based app by overriding CSS design tokens on
//! its embedded pages. Theme delivery supports both official apps:
//!
//! - [`live`]: inject at runtime over the Chrome DevTools Protocol on macOS and Windows
//! - [`build`]: macOS-only DoubaoWork offline clone + patch + re-sign into
//!   `~/Applications/DoubaoWork-Skin.app`

pub mod authoring;
pub mod auto_theme;
pub mod build;
pub mod live;
pub mod pak;
pub mod protocol_bridge;
pub mod theme;
mod theme_css;
pub mod theme_package;
pub mod ws;
