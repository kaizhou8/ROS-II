//! Service system for request-response communication
//! 
//! Provides ROS-style services for synchronous communication between nodes.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, oneshot, mpsc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use anyhow::{Result, anyhow};

/// Service request ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceRequestId(pub Uuid);

impl ServiceRequestId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ServiceRequestId {
    fn default() -> Self {
        Self::new()
    }
}

/// Service request trait
pub trait ServiceRequest: Send + Sync + std::fmt::Debug {
    type Response: ServiceResponse;
    
    fn service_name(&self) -> &str;
    fn request_id(&self) -> ServiceRequestId;
}

/// Service response trait
pub trait ServiceResponse: Send + Sync + std::fmt::Debug {
    fn success(&self) -> bool;
    fn error_message(&self) -> Option<&str>;
}

/// Service handler trait
#[async_trait::async_trait]
pub trait ServiceHandler<Req: ServiceRequest>: Send + Sync {
    async fn handle(&self, request: Req) -> Req::Response;
}

/// Service client for making requests
#[derive(Debug, Clone)]
pub struct ServiceClient {
    service_name: String,
    request_tx: mpsc::Sender<(Box<dyn std::any::Any + Send>, oneshot::Sender<Box<dyn std::any::Any + Send>>)>,
}

impl ServiceClient {
    pub fn new(
        service_name: String,
        request_tx: mpsc::Sender<(Box<dyn std::any::Any + Send>, oneshot::Sender<Box<dyn std::any::Any + Send>>)>,
    ) -> Self {
        Self {
            service_name,
            request_tx,
        }
    }
    
    pub async fn call<Req: ServiceRequest + 'static>(&self, request: Req) -> Result<Req::Response> {
        let (response_tx, response_rx) = oneshot::channel();
        
        self.request_tx.send((Box::new(request), response_tx)).await
            .map_err(|_| anyhow!("Service {} is not available", self.service_name))?;
        
        let response = response_rx.await
            .map_err(|_| anyhow!("Service {} did not respond", self.service_name))?;
        
        response.downcast::<Req::Response>()
            .map(|boxed| *boxed)
            .map_err(|_| anyhow!("Invalid response type from service {}", self.service_name))
    }
}

/// Service server for handling requests
#[derive(Debug)]
pub struct ServiceServer {
    services: Arc<RwLock<HashMap<String, mpsc::Sender<(Box<dyn std::any::Any + Send>, oneshot::Sender<Box<dyn std::any::Any + Send>>)>>>>,
}

impl ServiceServer {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn register_service<Req, H>(&self, service_name: String, handler: H) -> Result<()>
    where
        Req: ServiceRequest + 'static,
        H: ServiceHandler<Req> + 'static,
    {
        let (request_tx, mut request_rx) = mpsc::channel::<(Box<dyn std::any::Any + Send>, oneshot::Sender<Box<dyn std::any::Any + Send>>)>(100);
        
        // Store the sender for this service
        self.services.write().await.insert(service_name.clone(), request_tx);
        
        // Clone service_name for use in the spawned task
        let service_name_for_task = service_name.clone();
        
        // Spawn a task to handle requests
        tokio::spawn(async move {
            while let Some((request_any, response_tx)) = request_rx.recv().await {
                if let Ok(request) = request_any.downcast::<Req>() {
                    let response = handler.handle(*request).await;
                    let _ = response_tx.send(Box::new(response));
                } else {
                    log::error!("Invalid request type for service {}", service_name_for_task);
                }
            }
        });
        
        log::info!("Registered service: {}", service_name);
        Ok(())
    }
    
    pub async fn create_client(&self, service_name: &str) -> Result<ServiceClient> {
        let services = self.services.read().await;
        if let Some(request_tx) = services.get(service_name) {
            Ok(ServiceClient::new(service_name.to_string(), request_tx.clone()))
        } else {
            Err(anyhow!("Service {} not found", service_name))
        }
    }
    
    pub async fn list_services(&self) -> Vec<String> {
        let services = self.services.read().await;
        services.keys().cloned().collect()
    }
    
    pub async fn service_exists(&self, service_name: &str) -> bool {
        let services = self.services.read().await;
        services.contains_key(service_name)
    }
}

impl Default for ServiceServer {
    fn default() -> Self {
        Self::new()
    }
}

// Example service types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetParameterRequest {
    pub service_name: String,
    pub request_id: ServiceRequestId,
    pub parameter_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetParameterResponse {
    pub success: bool,
    pub error_message: Option<String>,
    pub parameter_value: Option<String>,
}

impl ServiceRequest for GetParameterRequest {
    type Response = GetParameterResponse;
    
    fn service_name(&self) -> &str {
        &self.service_name
    }
    
    fn request_id(&self) -> ServiceRequestId {
        self.request_id
    }
}

impl ServiceResponse for GetParameterResponse {
    fn success(&self) -> bool {
        self.success
    }
    
    fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParameterRequest {
    pub service_name: String,
    pub request_id: ServiceRequestId,
    pub parameter_name: String,
    pub parameter_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParameterResponse {
    pub success: bool,
    pub error_message: Option<String>,
}

impl ServiceRequest for SetParameterRequest {
    type Response = SetParameterResponse;
    
    fn service_name(&self) -> &str {
        &self.service_name
    }
    
    fn request_id(&self) -> ServiceRequestId {
        self.request_id
    }
}

impl ServiceResponse for SetParameterResponse {
    fn success(&self) -> bool {
        self.success
    }
    
    fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }
}