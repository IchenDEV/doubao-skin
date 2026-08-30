//! Desktop layout constants and inline icons.

pub const MAX_INTERNAL_LOGS: usize = 300;
pub const HEADER_HEIGHT: f32 = 72.0;
pub const TRAFFIC_LIGHT_X: f32 = 14.0;
pub const TRAFFIC_LIGHT_DIAMETER: f32 = 14.0;
pub const TRAFFIC_LIGHT_STEP: f32 = 20.0;
pub const TRAFFIC_LIGHT_Y: f32 = (HEADER_HEIGHT - TRAFFIC_LIGHT_DIAMETER) / 2.0;
pub const WINDOW_TITLE_GAP: f32 = 24.0;
pub const WINDOW_TITLE_X: f32 =
    TRAFFIC_LIGHT_X + TRAFFIC_LIGHT_STEP * 2.0 + TRAFFIC_LIGHT_DIAMETER + WINDOW_TITLE_GAP;
pub const PREVIEW_FRAME_RADIUS: f32 = 12.0;
pub const PREVIEW_CONTENT_RADIUS: f32 = PREVIEW_FRAME_RADIUS - 1.0;
pub const MIN_SURFACE_OPACITY: f32 = 0.35;
pub const SURFACE_OPACITY_RANGE: f32 = 0.65;
pub const OPACITY_TRACK_WIDTH: f32 = 180.0;
pub const MAIN_WINDOW_WIDTH: f32 = 1120.0;
pub const MAIN_WINDOW_HEIGHT: f32 = 720.0;

pub const SEARCH_ICON_SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="none"><circle cx="7" cy="7" r="4.25" stroke="currentColor" stroke-width="1.5"/><path d="M10.25 10.25 14 14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>"##;
pub const INSTALL_ICON_SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 18 18" fill="none"><path d="M3.25 6.25 9 3l5.75 3.25v6.5L9 16l-5.75-3.25v-6.5Z" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/><path d="M9 3v6.4m0 0 2.15-2.1M9 9.4 6.85 7.3" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/></svg>"##;
pub const REFRESH_ICON_SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="none"><path d="M13.1 6A5.3 5.3 0 1 0 13 10.3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><path d="M10.7 3.6h2.7v2.7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>"##;
