use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskState {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Paused,
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskState::Pending => write!(f, "PENDING"),
            TaskState::Running => write!(f, "RUNNING"),
            TaskState::Completed => write!(f, "COMPLETED"),
            TaskState::Failed => write!(f, "FAILED"),
            TaskState::Cancelled => write!(f, "CANCELLED"),
            TaskState::Paused => write!(f, "PAUSED"),
        }
    }
}

impl TaskState {
    pub fn is_terminal(&self) -> bool {
        matches!(self, TaskState::Completed | TaskState::Failed | TaskState::Cancelled)
    }
    
    pub fn is_active(&self) -> bool {
        matches!(self, TaskState::Running)
    }
    
    pub fn can_transition_to(&self, target: &TaskState) -> bool {
        use TaskState::*;
        
        match (self, target) {
            // From Pending
            (Pending, Running) => true,
            (Pending, Cancelled) => true,
            
            // From Running
            (Running, Completed) => true,
            (Running, Failed) => true,
            (Running, Cancelled) => true,
            (Running, Paused) => true,
            
            // From Paused
            (Paused, Running) => true,
            (Paused, Cancelled) => true,
            
            // From Failed (for retry)
            (Failed, Pending) => true,
            (Failed, Cancelled) => true,
            
            // No transitions from terminal states except Failed
            (Completed, _) => false,
            (Cancelled, _) => false,
            
            // Invalid transitions
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct StateManager {
    // Could be extended to track state transition history, metrics, etc.
}

impl StateManager {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn validate_transition(&self, current: &TaskState, target: &TaskState) -> Result<(), StateTransitionError> {
        if current.can_transition_to(target) {
            Ok(())
        } else {
            Err(StateTransitionError::InvalidTransition {
                from: current.clone(),
                to: target.clone(),
            })
        }
    }
    
    pub fn get_valid_transitions(&self, current: &TaskState) -> Vec<TaskState> {
        use TaskState::*;
        
        match current {
            Pending => vec![Running, Cancelled],
            Running => vec![Completed, Failed, Cancelled, Paused],
            Paused => vec![Running, Cancelled],
            Failed => vec![Pending, Cancelled], // Pending for retry
            Completed | Cancelled => vec![], // Terminal states
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StateTransitionError {
    #[error("Invalid state transition from {from} to {to}")]
    InvalidTransition { from: TaskState, to: TaskState },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_state_display() {
        assert_eq!(TaskState::Pending.to_string(), "PENDING");
        assert_eq!(TaskState::Running.to_string(), "RUNNING");
        assert_eq!(TaskState::Completed.to_string(), "COMPLETED");
        assert_eq!(TaskState::Failed.to_string(), "FAILED");
        assert_eq!(TaskState::Cancelled.to_string(), "CANCELLED");
        assert_eq!(TaskState::Paused.to_string(), "PAUSED");
    }
    
    #[test]
    fn test_terminal_states() {
        assert!(!TaskState::Pending.is_terminal());
        assert!(!TaskState::Running.is_terminal());
        assert!(!TaskState::Paused.is_terminal());
        assert!(TaskState::Completed.is_terminal());
        assert!(TaskState::Failed.is_terminal());
        assert!(TaskState::Cancelled.is_terminal());
    }
    
    #[test]
    fn test_active_states() {
        assert!(!TaskState::Pending.is_active());
        assert!(TaskState::Running.is_active());
        assert!(!TaskState::Paused.is_active());
        assert!(!TaskState::Completed.is_active());
        assert!(!TaskState::Failed.is_active());
        assert!(!TaskState::Cancelled.is_active());
    }
    
    #[test]
    fn test_valid_transitions() {
        // Pending transitions
        assert!(TaskState::Pending.can_transition_to(&TaskState::Running));
        assert!(TaskState::Pending.can_transition_to(&TaskState::Cancelled));
        assert!(!TaskState::Pending.can_transition_to(&TaskState::Completed));
        
        // Running transitions
        assert!(TaskState::Running.can_transition_to(&TaskState::Completed));
        assert!(TaskState::Running.can_transition_to(&TaskState::Failed));
        assert!(TaskState::Running.can_transition_to(&TaskState::Cancelled));
        assert!(TaskState::Running.can_transition_to(&TaskState::Paused));
        assert!(!TaskState::Running.can_transition_to(&TaskState::Pending));
        
        // Failed transitions (for retry)
        assert!(TaskState::Failed.can_transition_to(&TaskState::Pending));
        assert!(TaskState::Failed.can_transition_to(&TaskState::Cancelled));
        assert!(!TaskState::Failed.can_transition_to(&TaskState::Running));
        
        // Terminal state transitions
        assert!(!TaskState::Completed.can_transition_to(&TaskState::Running));
        assert!(!TaskState::Cancelled.can_transition_to(&TaskState::Pending));
    }
    
    #[test]
    fn test_state_manager() {
        let manager = StateManager::new();
        
        // Valid transition
        assert!(manager.validate_transition(&TaskState::Pending, &TaskState::Running).is_ok());
        
        // Invalid transition
        assert!(manager.validate_transition(&TaskState::Completed, &TaskState::Running).is_err());
        
        // Check valid transitions list
        let valid_from_running = manager.get_valid_transitions(&TaskState::Running);
        assert!(valid_from_running.contains(&TaskState::Completed));
        assert!(valid_from_running.contains(&TaskState::Failed));
        assert!(valid_from_running.contains(&TaskState::Cancelled));
        assert!(valid_from_running.contains(&TaskState::Paused));
        
        let valid_from_completed = manager.get_valid_transitions(&TaskState::Completed);
        assert!(valid_from_completed.is_empty());
    }
}