/// Design tokens extracted from a live page via getComputedStyle().
///
/// These are the actual computed values the browser uses — not the CSS source,
/// not approximations. This gives LLMs and cloners the exact values needed
/// for pixel-perfect reproduction.
use serde::{Deserialize, Serialize};

/// Full design token set extracted from a page.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DesignTokens {
    /// Source URL of the page.
    pub url: String,

    /// Page title.
    pub title: Option<String>,

    /// Color palette — unique colors used across the page, sorted by frequency.
    pub colors: ColorTokens,

    /// Typography — font families, sizes, weights used.
    pub typography: TypographyTokens,

    /// Spacing — padding, margin, gap values observed across key elements.
    pub spacing: SpacingTokens,

    /// Border radii — unique border-radius values found.
    pub border_radii: Vec<String>,

    /// Box shadows — unique box-shadow values found.
    pub shadows: Vec<String>,

    /// CSS custom properties (variables) from :root.
    pub css_variables: Vec<CssVariable>,

    /// Detected color scheme: "light", "dark", or "auto".
    pub color_scheme: String,

    /// Smooth scroll library detected (e.g., "lenis", "locomotive-scroll", "none").
    pub scroll_library: String,

    /// External font sources (Google Fonts URLs, self-hosted font-face declarations).
    pub font_sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColorTokens {
    /// Background colors, sorted by frequency.
    pub backgrounds: Vec<ColorEntry>,
    /// Text/foreground colors, sorted by frequency.
    pub foregrounds: Vec<ColorEntry>,
    /// Border colors.
    pub borders: Vec<ColorEntry>,
    /// Accent/highlight colors (buttons, links, focus rings).
    pub accents: Vec<ColorEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorEntry {
    /// Computed color value (e.g., "rgb(99, 102, 241)" or "rgba(0, 0, 0, 0.5)").
    pub value: String,
    /// Approximate hex for display (best-effort, ignores alpha).
    pub hex: Option<String>,
    /// Number of elements using this color.
    pub count: usize,
    /// Where this color was found: "background", "text", "border", "accent".
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TypographyTokens {
    /// All font families in use with their roles.
    pub families: Vec<FontFamily>,
    /// Font size scale (unique sizes, sorted smallest to largest).
    pub sizes: Vec<String>,
    /// Font weights in use.
    pub weights: Vec<String>,
    /// Line heights in use.
    pub line_heights: Vec<String>,
    /// Letter spacings in use (non-normal values only).
    pub letter_spacings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFamily {
    pub family: String,
    /// Where this font is used: "heading", "body", "code", "label", etc.
    pub roles: Vec<String>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpacingTokens {
    /// Unique padding values observed (non-zero).
    pub paddings: Vec<String>,
    /// Unique gap values in flex/grid containers.
    pub gaps: Vec<String>,
    /// Max-width values of containers.
    pub max_widths: Vec<String>,
    /// Section vertical padding values (useful for rhythm).
    pub section_paddings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CssVariable {
    pub name: String,
    pub value: String,
}
