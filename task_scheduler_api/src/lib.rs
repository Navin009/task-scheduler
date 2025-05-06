use anyhow::Result;

pub struct Server {
    // Add necessary fields here
}

impl Server {
    pub async fn new() -> Result<Self> {
        // Initialize the server
        Ok(Self {
            // Initialize fields
        })
    }

    pub async fn start(&self) -> Result<ServerHandle> {
        // Start the server and return a handle
        Ok(ServerHandle {
            // Initialize handle fields
        })
    }
}

pub struct ServerHandle {
    // Add necessary fields here
}

impl ServerHandle {
    pub async fn shutdown(&self) -> Result<()> {
        // Implement shutdown logic
        Ok(())
    }
}

pub mod config;
pub mod error;
pub mod guard;
pub mod model;
pub mod security;

pub use guard::api_key::ApiKeyGuard;
pub use model::auth::{AuthContext, Merchant, User};
