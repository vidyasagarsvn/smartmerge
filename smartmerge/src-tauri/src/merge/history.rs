use serde::{Serialize, Deserialize};
use super::state::MergeAction;

/// History entry for undo/redo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Block index affected
    pub block_index: usize,
    /// Previous action (for undo)
    pub previous_action: MergeAction,
    /// Previous custom lines (if any)
    pub previous_custom_lines: Option<Vec<String>>,
    /// New action applied
    pub new_action: MergeAction,
    /// New custom lines (if any)
    pub new_custom_lines: Option<Vec<String>>,
    /// Timestamp
    pub timestamp: u64,
    /// Description of the change
    pub description: String,
}

/// Undo/Redo history manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeHistory {
    /// Stack of completed actions
    history: Vec<HistoryEntry>,
    /// Current position in history
    current_position: usize,
    /// Maximum history size
    max_size: usize,
}

impl MergeHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            history: Vec::new(),
            current_position: 0,
            max_size,
        }
    }

    /// Add an action to history
    pub fn push(
        &mut self,
        block_index: usize,
        previous_action: MergeAction,
        previous_custom_lines: Option<Vec<String>>,
        new_action: MergeAction,
        new_custom_lines: Option<Vec<String>>,
        description: String,
    ) {
        // Remove any future history if we're not at the end
        if self.current_position < self.history.len() {
            self.history.truncate(self.current_position);
        }

        let entry = HistoryEntry {
            block_index,
            previous_action,
            previous_custom_lines,
            new_action,
            new_custom_lines,
            timestamp: current_timestamp(),
            description,
        };

        self.history.push(entry);
        self.current_position = self.history.len();

        // Limit history size
        if self.history.len() > self.max_size {
            self.history.remove(0);
            self.current_position -= 1;
        }
    }

    /// Undo the last action
    pub fn undo(&mut self) -> Option<&HistoryEntry> {
        if self.current_position == 0 {
            return None;
        }

        self.current_position -= 1;
        self.history.get(self.current_position)
    }

    /// Redo the next action
    pub fn redo(&mut self) -> Option<&HistoryEntry> {
        if self.current_position >= self.history.len() {
            return None;
        }

        let entry = self.history.get(self.current_position);
        self.current_position += 1;
        entry
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.current_position > 0
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.current_position < self.history.len()
    }

    /// Get current history position
    pub fn position(&self) -> usize {
        self.current_position
    }

    /// Get total history size
    pub fn size(&self) -> usize {
        self.history.len()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.history.clear();
        self.current_position = 0;
    }

    /// Get history at specific position
    pub fn get(&self, index: usize) -> Option<&HistoryEntry> {
        self.history.get(index)
    }

    /// Get all history entries
    pub fn entries(&self) -> &[HistoryEntry] {
        &self.history
    }
}

impl Default for MergeHistory {
    fn default() -> Self {
        Self::new(100) // Default max 100 history entries
    }
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_push_and_undo() {
        let mut history = MergeHistory::new(10);
        
        history.push(
            0,
            MergeAction::Pending,
            None,
            MergeAction::AcceptLeft,
            None,
            "Accept left version".to_string(),
        );

        assert_eq!(history.size(), 1);
        assert_eq!(history.position(), 1);
        assert!(history.can_undo());
        assert!(!history.can_redo());

        let entry = history.undo().unwrap();
        assert_eq!(entry.block_index, 0);
        assert_eq!(entry.new_action, MergeAction::AcceptLeft);
        assert!(history.can_redo());
    }

    #[test]
    fn test_history_redo() {
        let mut history = MergeHistory::new(10);
        
        history.push(
            0,
            MergeAction::Pending,
            None,
            MergeAction::AcceptLeft,
            None,
            "Test".to_string(),
        );

        history.undo();
        assert!(history.can_redo());

        let entry = history.redo().unwrap();
        assert_eq!(entry.new_action, MergeAction::AcceptLeft);
        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_size_limit() {
        let mut history = MergeHistory::new(3);
        
        for i in 0..5 {
            history.push(
                i,
                MergeAction::Pending,
                None,
                MergeAction::AcceptLeft,
                None,
                format!("Action {}", i),
            );
        }

        assert_eq!(history.size(), 3);
    }
}
