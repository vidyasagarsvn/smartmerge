use serde::{Serialize, Deserialize};

/// Whitespace handling mode for diff
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WhitespaceMode {
    /// Compare all whitespace (default)
    None,
    /// Ignore changes in amount of whitespace
    IgnoreChange,
    /// Ignore all whitespace
    IgnoreAll,
    /// Ignore trailing whitespace
    IgnoreTrailing,
}

impl Default for WhitespaceMode {
    fn default() -> Self {
        Self::None
    }
}

/// Options for diff filtering and comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffOptions {
    /// Ignore case when comparing lines
    pub ignore_case: bool,
    
    /// Whitespace handling mode
    pub whitespace_mode: WhitespaceMode,
    
    /// Ignore blank lines
    pub ignore_blank_lines: bool,
    
    /// Ignore comment lines (simple heuristic)
    pub ignore_comments: bool,
    
    /// Context lines to show around differences
    pub context_lines: usize,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            ignore_case: false,
            whitespace_mode: WhitespaceMode::None,
            ignore_blank_lines: false,
            ignore_comments: false,
            context_lines: 3,
        }
    }
}

/// Apply filters to input lines based on diff options
pub fn apply_filters(left: &[String], right: &[String], options: &DiffOptions) -> (Vec<String>, Vec<String>) {
    let filtered_left = left.iter().map(|line| filter_line(line, options)).collect();
    let filtered_right = right.iter().map(|line| filter_line(line, options)).collect();
    (filtered_left, filtered_right)
}

/// Apply filters to a single line
fn filter_line(line: &str, options: &DiffOptions) -> String {
    let mut result = line.to_string();
    
    // Handle whitespace filtering
    match options.whitespace_mode {
        WhitespaceMode::None => {},
        WhitespaceMode::IgnoreChange => {
            // Normalize all sequences of whitespace to single space
            result = normalize_whitespace(&result);
        },
        WhitespaceMode::IgnoreAll => {
            // Remove all whitespace
            result = result.chars().filter(|c| !c.is_whitespace()).collect();
        },
        WhitespaceMode::IgnoreTrailing => {
            // Remove trailing whitespace
            result = result.trim_end().to_string();
        },
    }
    
    // Handle case insensitivity
    if options.ignore_case {
        result = result.to_lowercase();
    }
    
    result
}

/// Normalize whitespace sequences to single spaces
fn normalize_whitespace(s: &str) -> String {
    let mut result = String::new();
    let mut prev_ws = false;
    
    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_ws {
                result.push(' ');
                prev_ws = true;
            }
        } else {
            result.push(c);
            prev_ws = false;
        }
    }
    
    result
}

/// Check if a line should be ignored based on filters
pub fn should_ignore_line(line: &str, options: &DiffOptions) -> bool {
    let trimmed = line.trim();
    
    // Check blank lines
    if options.ignore_blank_lines && trimmed.is_empty() {
        return true;
    }
    
    // Check comments (simple heuristic)
    if options.ignore_comments {
        if trimmed.starts_with("//") 
            || trimmed.starts_with("#") 
            || trimmed.starts_with("/*") 
            || trimmed.starts_with("*") 
            || trimmed.starts_with("<!--")
        {
            return true;
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(normalize_whitespace("hello  world"), "hello world");
        assert_eq!(normalize_whitespace("hello\t\tworld"), "hello world");
        assert_eq!(normalize_whitespace("  hello  "), " hello ");
    }
    
    #[test]
    fn test_filter_line_ignore_case() {
        let options = DiffOptions {
            ignore_case: true,
            ..Default::default()
        };
        assert_eq!(filter_line("Hello World", &options), "hello world");
    }
    
    #[test]
    fn test_filter_line_ignore_all_whitespace() {
        let options = DiffOptions {
            whitespace_mode: WhitespaceMode::IgnoreAll,
            ..Default::default()
        };
        assert_eq!(filter_line("hello world", &options), "helloworld");
    }
    
    #[test]
    fn test_filter_line_ignore_trailing() {
        let options = DiffOptions {
            whitespace_mode: WhitespaceMode::IgnoreTrailing,
            ..Default::default()
        };
        assert_eq!(filter_line("hello world  ", &options), "hello world");
    }
    
    #[test]
    fn test_should_ignore_blank_line() {
        let options = DiffOptions {
            ignore_blank_lines: true,
            ..Default::default()
        };
        assert!(should_ignore_line("", &options));
        assert!(should_ignore_line("   ", &options));
        assert!(!should_ignore_line("hello", &options));
    }
    
    #[test]
    fn test_should_ignore_comments() {
        let options = DiffOptions {
            ignore_comments: true,
            ..Default::default()
        };
        assert!(should_ignore_line("// comment", &options));
        assert!(should_ignore_line("# comment", &options));
        assert!(should_ignore_line("/* comment */", &options));
        assert!(!should_ignore_line("hello", &options));
    }
}
