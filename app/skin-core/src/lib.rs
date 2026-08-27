//! skin-core: Rust port of the `doubao_skin` Python package.
//!
//! Reskins the DoubaoWork macOS app (Chromium-based) by forcing
//! `data-theme="dark"` and overriding CSS design tokens on every embedded
//! page. Two delivery modes:
//!
//! - [`live`]: inject at runtime over the Chrome DevTools Protocol
//! - [`build`]: offline clone + patch + re-sign into
//!   `~/Applications/DoubaoWork-Skin.app`

pub mod build;
pub mod live;
pub mod pak;
pub mod theme;
pub mod ws;
