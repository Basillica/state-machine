use std::fmt;
use std::error::Error;

/// Custom error that can be thrown at any point in the execution
#[derive(Debug)]
pub struct StateMachineError {
    /// error string
    pub message: String,
}

impl fmt::Display for StateMachineError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.message)
  }
}

impl Error for StateMachineError {}