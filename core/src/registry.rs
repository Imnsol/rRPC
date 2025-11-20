//! Function registry for RPC handlers

use crate::error::RpcError;
use std::collections::HashMap;

/// Handler function type: input bytes → Result<output bytes, error>
pub type Handler = Box<dyn Fn(&[u8]) -> Result<Vec<u8>, RpcError> + Send + Sync>;

/// Registry of RPC method handlers
pub struct Registry {
    handlers: HashMap<String, Handler>,
}

impl Registry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a handler function for a method name
    ///
    /// # Example
    /// ```
    /// use rrpc_core::{Registry, RpcError};
    ///
    /// let mut registry = Registry::new();
    /// registry.register("echo", |input| Ok(input.to_vec()));
    /// ```
    pub fn register<F>(&mut self, name: impl Into<String>, handler: F)
    where
        F: Fn(&[u8]) -> Result<Vec<u8>, RpcError> + Send + Sync + 'static,
    {
        self.handlers.insert(name.into(), Box::new(handler));
    }

    /// Call a registered method
    pub fn call(&self, method: &str, input: &[u8]) -> Result<Vec<u8>, RpcError> {
        let handler = self
            .handlers
            .get(method)
            .ok_or_else(|| RpcError::UnknownMethod(method.to_string()))?;

        handler(input)
    }

    /// Check if a method is registered
    pub fn has_method(&self, method: &str) -> bool {
        self.handlers.contains_key(method)
    }

    /// Get list of all registered methods
    pub fn methods(&self) -> Vec<&str> {
        self.handlers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_call() {
        let mut registry = Registry::new();
        
        registry.register("echo", |input| Ok(input.to_vec()));
        
        let result = registry.call("echo", b"test").unwrap();
        assert_eq!(result, b"test");
    }

    #[test]
    fn test_unknown_method() {
        let registry = Registry::new();
        
        let result = registry.call("missing", b"test");
        assert!(matches!(result, Err(RpcError::UnknownMethod(_))));
    }

    #[test]
    fn test_has_method() {
        let mut registry = Registry::new();
        
        registry.register("test", |_| Ok(vec![]));
        
        assert!(registry.has_method("test"));
        assert!(!registry.has_method("missing"));
    }
}
