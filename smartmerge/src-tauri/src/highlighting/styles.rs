use serde::{Serialize, Deserialize};
use crate::models::BlockType;

/// Theme type for highlighting
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    Auto,
}

impl Default for Theme {
    fn default() -> Self {
        Self::Auto
    }
}

/// Color configuration for a specific change type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeTypeColors {
    /// Background color (normal state)
    pub background: String,
    /// Text color (normal state)
    pub text: String,
    /// Background color (selected state)
    pub selected_background: Option<String>,
    /// Text color (selected state)
    pub selected_text: Option<String>,
    /// Border color
    pub border: Option<String>,
}

impl ChangeTypeColors {
    pub fn new(background: &str, text: &str) -> Self {
        Self {
            background: background.to_string(),
            text: text.to_string(),
            selected_background: None,
            selected_text: None,
            border: None,
        }
    }

    pub fn with_selected(mut self, bg: &str, text: &str) -> Self {
        self.selected_background = Some(bg.to_string());
        self.selected_text = Some(text.to_string());
        self
    }

    pub fn with_border(mut self, border: &str) -> Self {
        self.border = Some(border.to_string());
        self
    }
}

/// Color scheme for diff highlighting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    /// Name of the color scheme
    pub name: String,
    /// Version/ID of the color scheme
    pub version: String,
    
    /// Colors for added lines
    pub additions: ChangeTypeColors,
    /// Colors for deleted lines
    pub deletions: ChangeTypeColors,
    /// Colors for modified lines
    pub modifications: ChangeTypeColors,
    /// Colors for moved blocks
    pub moved_blocks: ChangeTypeColors,
    /// Colors for trivial changes (whitespace-only)
    pub trivial_changes: ChangeTypeColors,
    /// Colors for equal/unchanged lines
    pub equal: ChangeTypeColors,
    /// Colors for conflict markers
    pub conflicts: ChangeTypeColors,
    
    /// Inline diff highlight background
    pub inline_highlight_bg: String,
    /// Inline diff highlight text color
    pub inline_highlight_text: String,
    /// General border color
    pub border_color: String,
    /// Background color for the editor
    pub background: String,
    /// Default text color
    pub text_color: String,
}

impl ColorScheme {
    /// Get light theme color scheme (GitHub style)
    pub fn light() -> Self {
        Self {
            name: "GitHub Light".to_string(),
            version: "1.0".to_string(),
            additions: ChangeTypeColors::new("#d4f4dd", "#0d5a0d")
                .with_selected("#c6e9c3", "#0d5a0d")
                .with_border("#34a853"),
            deletions: ChangeTypeColors::new("#ffd7d5", "#82071e")
                .with_selected("#ffc4bf", "#82071e")
                .with_border("#da3633"),
            modifications: ChangeTypeColors::new("#fff5b1", "#7d4e00")
                .with_selected("#fffcc9", "#7d4e00")
                .with_border("#bf8700"),
            moved_blocks: ChangeTypeColors::new("#d8e5ff", "#0550ae")
                .with_selected("#c9d9f0", "#0550ae")
                .with_border("#0969da"),
            trivial_changes: ChangeTypeColors::new("#f6f8fa", "#57606a")
                .with_selected("#eaeef2", "#57606a"),
            equal: ChangeTypeColors::new("#ffffff", "#24292f"),
            conflicts: ChangeTypeColors::new("#ffe5e5", "#d73a49")
                .with_border("#d73a49"),
            inline_highlight_bg: "#a8f0a8".to_string(),
            inline_highlight_text: "#0d5a0d".to_string(),
            border_color: "#d0d7de".to_string(),
            background: "#ffffff".to_string(),
            text_color: "#24292f".to_string(),
        }
    }

    /// Get dark theme color scheme (GitHub Dark style)
    pub fn dark() -> Self {
        Self {
            name: "GitHub Dark".to_string(),
            version: "1.0".to_string(),
            additions: ChangeTypeColors::new("#1a4d2e", "#7ee787")
                .with_selected("#0d3219", "#7ee787")
                .with_border("#3fb950"),
            deletions: ChangeTypeColors::new("#5a1e1e", "#ff7b72")
                .with_selected("#42111a", "#ff7b72")
                .with_border("#f85149"),
            modifications: ChangeTypeColors::new("#5a4a1a", "#f0c932")
                .with_selected("#3d2d0d", "#f0c932")
                .with_border("#d29922"),
            moved_blocks: ChangeTypeColors::new("#1a3a5a", "#79c0ff")
                .with_selected("#0d2438", "#79c0ff")
                .with_border("#58a6ff"),
            trivial_changes: ChangeTypeColors::new("#161b22", "#8b949e")
                .with_selected("#0d1117", "#8b949e"),
            equal: ChangeTypeColors::new("#0d1117", "#e6edf3"),
            conflicts: ChangeTypeColors::new("#5a1a1a", "#ff7b72")
                .with_border("#f85149"),
            inline_highlight_bg: "#2d5a2d".to_string(),
            inline_highlight_text: "#7ee787".to_string(),
            border_color: "#30363d".to_string(),
            background: "#0d1117".to_string(),
            text_color: "#e6edf3".to_string(),
        }
    }

    /// Bitbucket color scheme
    pub fn bitbucket() -> Self {
        Self {
            name: "Bitbucket".to_string(),
            version: "1.0".to_string(),
            additions: ChangeTypeColors::new("#cfffd6", "#0c3914")
                .with_selected("#aef5cd", "#0c3914")
                .with_border("#34d399"),
            deletions: ChangeTypeColors::new("#ffcccc", "#5c2d2d")
                .with_selected("#ffb3b3", "#5c2d2d")
                .with_border("#ef5350"),
            modifications: ChangeTypeColors::new("#fff8d6", "#664d00")
                .with_selected("#ffe680", "#664d00")
                .with_border("#ffa500"),
            moved_blocks: ChangeTypeColors::new("#d4e8ff", "#003366")
                .with_selected("#b3d9ff", "#003366")
                .with_border("#0066cc"),
            trivial_changes: ChangeTypeColors::new("#f5f5f5", "#666666")
                .with_selected("#ebebeb", "#666666"),
            equal: ChangeTypeColors::new("#ffffff", "#333333"),
            conflicts: ChangeTypeColors::new("#ffeeee", "#cc0000")
                .with_border("#cc0000"),
            inline_highlight_bg: "#ccffcc".to_string(),
            inline_highlight_text: "#0c3914".to_string(),
            border_color: "#dddddd".to_string(),
            background: "#ffffff".to_string(),
            text_color: "#333333".to_string(),
        }
    }

    /// Solarized light color scheme
    pub fn solarized_light() -> Self {
        Self {
            name: "Solarized Light".to_string(),
            version: "1.0".to_string(),
            additions: ChangeTypeColors::new("#e4f4dd", "#558a00")
                .with_selected("#d0e8c5", "#558a00")
                .with_border("#859900"),
            deletions: ChangeTypeColors::new("#f4dcd0", "#d33682")
                .with_selected("#e8b4a0", "#d33682")
                .with_border("#dc322f"),
            modifications: ChangeTypeColors::new("#fffacd", "#b58900")
                .with_selected("#fff8cc", "#b58900")
                .with_border("#b58900"),
            moved_blocks: ChangeTypeColors::new("#d1dff0", "#0066cc")
                .with_selected("#b5c9e8", "#0066cc")
                .with_border("#268bd2"),
            trivial_changes: ChangeTypeColors::new("#eee8d5", "#657b83")
                .with_selected("#e8dcc2", "#657b83"),
            equal: ChangeTypeColors::new("#fdf6e3", "#657b83"),
            conflicts: ChangeTypeColors::new("#f0e5d0", "#dc322f")
                .with_border("#dc322f"),
            inline_highlight_bg: "#d5f0aa".to_string(),
            inline_highlight_text: "#558a00".to_string(),
            border_color: "#ddd8c4".to_string(),
            background: "#fdf6e3".to_string(),
            text_color: "#657b83".to_string(),
        }
    }

    /// Solarized dark color scheme
    pub fn solarized_dark() -> Self {
        Self {
            name: "Solarized Dark".to_string(),
            version: "1.0".to_string(),
            additions: ChangeTypeColors::new("#2d5016", "#859900")
                .with_selected("#1c3d0a", "#859900")
                .with_border("#859900"),
            deletions: ChangeTypeColors::new("#5a1f2f", "#dc322f")
                .with_selected("#3d1220", "#dc322f")
                .with_border("#dc322f"),
            modifications: ChangeTypeColors::new("#4a4a1a", "#b58900")
                .with_selected("#33330d", "#b58900")
                .with_border("#b58900"),
            moved_blocks: ChangeTypeColors::new("#1a3a5a", "#268bd2")
                .with_selected("#0d2438", "#268bd2")
                .with_border("#268bd2"),
            trivial_changes: ChangeTypeColors::new("#073642", "#839496")
                .with_selected("#002b36", "#839496"),
            equal: ChangeTypeColors::new("#002b36", "#839496"),
            conflicts: ChangeTypeColors::new("#5a1a2f", "#dc322f")
                .with_border("#dc322f"),
            inline_highlight_bg: "#3d5616".to_string(),
            inline_highlight_text: "#859900".to_string(),
            border_color: "#173340".to_string(),
            background: "#002b36".to_string(),
            text_color: "#839496".to_string(),
        }
    }

    /// VSCode light theme color scheme
    pub fn vscode_light() -> Self {
        Self {
            name: "VS Code Light".to_string(),
            version: "1.0".to_string(),
            additions: ChangeTypeColors::new("#d7fcd6", "#1a8f1a")
                .with_selected("#c1f0be", "#1a8f1a")
                .with_border("#24a24a"),
            deletions: ChangeTypeColors::new("#ffd6e5", "#a1184f")
                .with_selected("#ffb8ce", "#a1184f")
                .with_border("#f04872"),
            modifications: ChangeTypeColors::new("#fff5d6", "#8a5300")
                .with_selected("#fff0ad", "#8a5300")
                .with_border("#d9a934"),
            moved_blocks: ChangeTypeColors::new("#d4e5ff", "#0066cc")
                .with_selected("#c0d9ff", "#0066cc")
                .with_border("#0066ff"),
            trivial_changes: ChangeTypeColors::new("#f3f3f3", "#666666")
                .with_selected("#ececec", "#666666"),
            equal: ChangeTypeColors::new("#ffffff", "#333333"),
            conflicts: ChangeTypeColors::new("#ffe5eb", "#e81828")
                .with_border("#e81828"),
            inline_highlight_bg: "#ccffcc".to_string(),
            inline_highlight_text: "#1a8f1a".to_string(),
            border_color: "#d9d9d9".to_string(),
            background: "#ffffff".to_string(),
            text_color: "#333333".to_string(),
        }
    }

    /// VSCode dark theme color scheme
    pub fn vscode_dark() -> Self {
        Self {
            name: "VS Code Dark".to_string(),
            version: "1.0".to_string(),
            additions: ChangeTypeColors::new("#1a3e1a", "#85cc00")
                .with_selected("#0d260d", "#85cc00")
                .with_border("#85cc00"),
            deletions: ChangeTypeColors::new("#3f1f2f", "#f48771")
                .with_selected("#2a1520", "#f48771")
                .with_border("#f48771"),
            modifications: ChangeTypeColors::new("#4a4a00", "#dcdcaa")
                .with_selected("#333300", "#dcdcaa")
                .with_border("#dcdcaa"),
            moved_blocks: ChangeTypeColors::new("#1a3a5a", "#569cd6")
                .with_selected("#0d2438", "#569cd6")
                .with_border("#569cd6"),
            trivial_changes: ChangeTypeColors::new("#1e1e1e", "#858585")
                .with_selected("#161616", "#858585"),
            equal: ChangeTypeColors::new("#1e1e1e", "#d4d4d4"),
            conflicts: ChangeTypeColors::new("#3f1f1f", "#f48771")
                .with_border("#f48771"),
            inline_highlight_bg: "#384a47".to_string(),
            inline_highlight_text: "#85cc00".to_string(),
            border_color: "#3e3e42".to_string(),
            background: "#1e1e1e".to_string(),
            text_color: "#d4d4d4".to_string(),
        }
    }

    /// Get color scheme by name
    pub fn get_scheme(name: &str) -> Option<Self> {
        match name {
            "github-light" => Some(Self::light()),
            "github-dark" => Some(Self::dark()),
            "bitbucket" => Some(Self::bitbucket()),
            "solarized-light" => Some(Self::solarized_light()),
            "solarized-dark" => Some(Self::solarized_dark()),
            "vscode-light" => Some(Self::vscode_light()),
            "vscode-dark" => Some(Self::vscode_dark()),
            _ => None,
        }
    }

    /// List all available color schemes
    pub fn available_schemes() -> Vec<(String, String)> {
        vec![
            ("github-light".to_string(), "GitHub Light".to_string()),
            ("github-dark".to_string(), "GitHub Dark".to_string()),
            ("bitbucket".to_string(), "Bitbucket".to_string()),
            ("solarized-light".to_string(), "Solarized Light".to_string()),
            ("solarized-dark".to_string(), "Solarized Dark".to_string()),
            ("vscode-light".to_string(), "VS Code Light".to_string()),
            ("vscode-dark".to_string(), "VS Code Dark".to_string()),
        ]
    }

    /// Get color scheme for theme
    pub fn for_theme(theme: Theme) -> Self {
        match theme {
            Theme::Light => Self::light(),
            Theme::Dark => Self::dark(),
            Theme::Auto => Self::light(), // Default to light, can be system-aware
        }
    }

    /// Get colors for a specific change type
    pub fn get_colors_for_type(&self, block_type: &BlockType) -> &ChangeTypeColors {
        match block_type {
            BlockType::Insert => &self.additions,
            BlockType::Delete => &self.deletions,
            BlockType::Replace => &self.modifications,
            BlockType::Moved => &self.moved_blocks,
            BlockType::Equal => &self.equal,
        }
    }
}

/// Highlight style for a line or region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightStyle {
    /// Background color
    pub background: String,
    /// Foreground/text color
    pub foreground: String,
    /// Whether the line is highlighted
    pub highlighted: bool,
    /// Whether to show a gutter indicator
    pub show_gutter: bool,
    /// Gutter indicator color
    pub gutter_color: Option<String>,
    /// CSS classes to apply
    pub css_classes: Vec<String>,
    /// Border style
    pub border: Option<String>,
}

impl HighlightStyle {
    pub fn new(background: String, foreground: String) -> Self {
        Self {
            background,
            foreground,
            highlighted: true,
            show_gutter: true,
            gutter_color: None,
            css_classes: Vec::new(),
            border: None,
        }
    }

    pub fn with_classes(mut self, classes: Vec<String>) -> Self {
        self.css_classes = classes;
        self
    }

    pub fn with_gutter(mut self, color: String) -> Self {
        self.gutter_color = Some(color);
        self
    }

    pub fn with_border(mut self, border: String) -> Self {
        self.border = Some(border);
        self
    }
}

/// Line highlight information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineHighlight {
    /// Line number (0-based)
    pub line_number: usize,
    /// Highlight style
    pub style: HighlightStyle,
    /// Block type
    pub block_type: BlockType,
    /// Inline highlights (for word-level diffs)
    pub inline_highlights: Vec<InlineHighlight>,
    /// Whether this line is part of a conflict
    pub is_conflict: bool,
}

/// Inline/word-level highlight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineHighlight {
    /// Start column (0-based)
    pub start: usize,
    /// End column (exclusive)
    pub end: usize,
    /// Background color
    pub background: String,
}

/// Complete highlight result for a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightResult {
    /// Highlights for left side
    pub left_highlights: Vec<LineHighlight>,
    /// Highlights for right side
    pub right_highlights: Vec<LineHighlight>,
    /// Color scheme used
    pub color_scheme: ColorScheme,
    /// Theme used
    pub theme: Theme,
}

impl HighlightResult {
    pub fn new(theme: Theme) -> Self {
        Self {
            left_highlights: Vec::new(),
            right_highlights: Vec::new(),
            color_scheme: ColorScheme::for_theme(theme),
            theme,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_scheme_light() {
        let scheme = ColorScheme::light();
        assert!(!scheme.additions.background.is_empty());
        assert!(!scheme.deletions.background.is_empty());
    }

    #[test]
    fn test_color_scheme_dark() {
        let scheme = ColorScheme::dark();
        assert!(!scheme.additions.background.is_empty());
        assert!(!scheme.deletions.background.is_empty());
    }

    #[test]
    fn test_highlight_style_builder() {
        let style = HighlightStyle::new("#ffffff".to_string(), "#000000".to_string())
            .with_classes(vec!["diff-added".to_string()])
            .with_gutter("#00ff00".to_string());
        
        assert_eq!(style.background, "#ffffff");
        assert_eq!(style.css_classes.len(), 1);
        assert!(style.gutter_color.is_some());
    }
}
