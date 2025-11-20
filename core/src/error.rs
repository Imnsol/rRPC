//! Error types for rRPC

use std::fmt;

/// RPC error types
#[derive(Debug, Clone)]
pub enum RpcError {
    /// Method not found in registry
    UnknownMethod(String),
    
    /// Resource not found (e.g., user ID)
    NotFound(String),
    
    /// Failed to parse input
    ParseError(String),
    
    /// Failed to serialize output
    SerializationError(String),
    
    /// Internal error
    Internal(String),
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RpcError::UnknownMethod(m) => write!(f, "Unknown method: {}", m),
            RpcError::NotFound(r) => write!(f, "Not found: {}", r),
            RpcError::ParseError(e) => write!(f, "Parse error: {}", e),
            RpcError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            RpcError::Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl std::error::Error for RpcError {}
