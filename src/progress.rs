//! Progress callback traits for compression and extraction operations
//! 
//! This module provides traits for monitoring progress and cancelling operations.

use std::sync::atomic::{AtomicBool, Ordering};

/// Progress callback trait for monitoring operations
pub trait ProgressCallback: Send + Sync {
    /// Called when total size is known
    fn on_total(&mut self, total: u64);

    /// Called when progress is made
    fn on_completed(&mut self, completed: u64);

    /// Check if operation should be cancelled
    fn should_cancel(&self) -> bool {
        false
    }
}

/// Simple progress callback implementation

pub struct SimpleProgress {
    total: u64,
    completed: u64,
    cancelled: AtomicBool,
}

impl SimpleProgress {
    /// Create new simple progress tracker
    pub fn new() -> Self {
        SimpleProgress {
            total: 0,
            completed: 0,
            cancelled: AtomicBool::new(false),
        }
    }
    
    /// Get total size (if known)
    pub fn total(&self) -> u64 {
        self.total
    }
    
    /// Get current completed amount
    pub fn completed(&self) -> u64 {
        self.completed
    }
    
    /// Get progress percentage (0-100)
    pub fn percentage(&self) -> Option<f64> {
        let total = self.total;
        if total == 0 {
            return Some(100.0);
        }
        let completed = self.completed;
        Some((completed as f64 / total as f64) * 100.0)
    }
    
    /// Cancel the operation
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
    
    /// Check if operation is cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl ProgressCallback for SimpleProgress {
    fn on_total(&mut self, total: u64) {
        self.total = total;
    }

    fn on_completed(&mut self, completed: u64) {
        self.completed = completed;
    }

    fn should_cancel(&self) -> bool {
        self.is_cancelled()
    }
}

impl Default for SimpleProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_progress() {
        let mut progress = SimpleProgress::new();
        
        progress.on_total(1000);
        assert_eq!(progress.total(), 1000);
        
        progress.on_completed(500);
        assert_eq!(progress.completed(), 500);
        assert_eq!(progress.percentage(), Some(50.0));
        
        assert!(!progress.should_cancel());
        progress.cancel();
        assert!(progress.should_cancel());
    }
}
