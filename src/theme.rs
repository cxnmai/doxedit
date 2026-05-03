use gpui::{Rgba, rgb};

#[derive(Clone, Copy)]
pub struct AppColors {
    pub background: Rgba,
    pub surface: Rgba,
    pub surface_elevated: Rgba,
    pub border: Rgba,
    pub text: Rgba,
    pub text_muted: Rgba,
    pub accent: Rgba,
}

impl AppColors {
    fn light() -> Self {
        Self {
            background: rgb(0xf7f8fa),
            surface: rgb(0xffffff),
            surface_elevated: rgb(0xf0f3f6),
            border: rgb(0xd7dce2),
            text: rgb(0x1f2937),
            text_muted: rgb(0x667085),
            accent: rgb(0x2563eb),
        }
    }

    fn dark() -> Self {
        Self {
            background: rgb(0x242629),
            surface: rgb(0x2d3035),
            surface_elevated: rgb(0x383c42),
            border: rgb(0x4a4f57),
            text: rgb(0xf4f6f8),
            text_muted: rgb(0xb5bdc8),
            accent: rgb(0x8ab4ff),
        }
    }
}

#[derive(Clone, Copy)]
pub enum ColorMode {
    Light,
    Dark,
}

impl ColorMode {
    pub fn initial() -> Self {
        match std::env::var("DEBATEDITOR_THEME") {
            Ok(theme) if theme.eq_ignore_ascii_case("dark") => Self::Dark,
            _ => Self::Light,
        }
    }

    pub fn colors(self) -> AppColors {
        match self {
            Self::Light => AppColors::light(),
            Self::Dark => AppColors::dark(),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}
