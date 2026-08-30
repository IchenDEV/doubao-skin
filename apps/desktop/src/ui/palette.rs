//! Appearance-aware desktop color palette.

use gpui::WindowAppearance;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UiPalette {
    pub shell: u32,
    pub sidebar: u32,
    pub control: u32,
    pub text: u32,
    pub muted: u32,
    pub border: u32,
    pub hover: u32,
    pub danger: u32,
    pub focus_border: u32,
    pub segmented_track: u32,
    pub segmented_selected: u32,
    pub drop_border: u32,
    pub drop_hover: u32,
    pub drop_accent: u32,
    pub link: u32,
    pub preview_placeholder: u32,
    pub installed_control: u32,
    pub card_hover_border: u32,
    pub slider_accent: u32,
}

impl UiPalette {
    pub fn for_appearance(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self {
                shell: 0x1c1c1e,
                sidebar: 0x242426,
                control: 0x2c2c2e,
                text: 0xf2f2f7,
                muted: 0xa7a7ad,
                border: 0x3a3a3c,
                hover: 0x363638,
                danger: 0xff7b79,
                focus_border: 0xc88d70,
                segmented_track: 0x29292b,
                segmented_selected: 0x48484a,
                drop_border: 0x555558,
                drop_hover: 0x3a2c28,
                drop_accent: 0xe0926e,
                link: 0xe0926e,
                preview_placeholder: 0x262628,
                installed_control: 0x3a3a3c,
                card_hover_border: 0x66666a,
                slider_accent: 0xc58b70,
            },
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self {
                shell: 0xf8f8f7,
                sidebar: 0xf1f2f3,
                control: 0xffffff,
                text: 0x242321,
                muted: 0x74726e,
                border: 0xdadbdc,
                hover: 0xe7e8e9,
                danger: 0xa84b4b,
                focus_border: 0x9c7b6b,
                segmented_track: 0xeeeeed,
                segmented_selected: 0xffffff,
                drop_border: 0xb9b9b7,
                drop_hover: 0xf1e8e3,
                drop_accent: 0xa64e24,
                link: 0x9d4a24,
                preview_placeholder: 0xf0f1f2,
                installed_control: 0xeeeeed,
                card_hover_border: 0xbcbdbc,
                slider_accent: 0x8f6b5b,
            },
        }
    }
}
