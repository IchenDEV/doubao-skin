//! Embedded UI assets.

use std::borrow::Cow;
use std::path::Path;

use gpui::{AssetSource, ImageSource, SharedString};

use crate::ui::constants::{INSTALL_ICON_SVG, REFRESH_ICON_SVG, SEARCH_ICON_SVG};

pub struct Assets;

pub(crate) fn local_image_source(path: &Path) -> ImageSource {
    path.into()
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        let embedded = match path {
            "icons/search.svg" => Some(SEARCH_ICON_SVG),
            "icons/install.svg" => Some(INSTALL_ICON_SVG),
            "icons/refresh.svg" => Some(REFRESH_ICON_SVG),
            _ => None,
        };
        if let Some(bytes) = embedded {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        Ok(Some(std::fs::read(path)?.into()))
    }

    fn list(&self, path: &str) -> anyhow::Result<Vec<SharedString>> {
        Ok(std::fs::read_dir(path)?
            .filter_map(|entry| {
                Some(SharedString::from(
                    entry.ok()?.path().to_string_lossy().into_owned(),
                ))
            })
            .collect())
    }
}
