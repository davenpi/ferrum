use serde::{Deserialize, Serialize};
use std::future::Future;

// Core trait that all services must implement
pub trait Service: Send + Sync + Serialize + for<'de> Deserialize<'de> {
    type Error: Send + Sync + std::error::Error + 'static;
    
    // Service lifecycle
    fn service_name() -> &'static str;
    
    // Method dispatch - services implement this to handle method calls
    fn call_method(
        &mut self,
        method_name: &str,
        args: Vec<u8>, // Serialized arguments
    ) -> impl Future<Output = Result<Vec<u8>, Self::Error>> + Send; // Serialized result
}

// Service metadata for registration
pub struct ServiceDefinition {
    pub name: &'static str,
    pub methods: Vec<MethodDefinition>,
}

pub struct MethodDefinition {
    pub name: &'static str,
    pub is_mutable: bool,
    // Future: parameter types, return type for validation
}
