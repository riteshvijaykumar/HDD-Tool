use reqwest;
use serde_json;
use uuid::Uuid;
use crate::server::models::*;

pub struct ServerClient {
    client: reqwest::Client,
    base_url: String,
    token: Option<String>,
    user_id: Option<Uuid>,
}

impl ServerClient {
    pub fn new(server_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: server_url.to_string(),
            token: None,
            user_id: None,
        }
    }
    
    pub async fn register(&mut self, username: &str, email: &str, password: &str) -> Result<LoginResponse, Box<dyn std::error::Error + Send + Sync>> {
        let req = CreateUserRequest {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        };
        
        let response = self.client
            .post(&format!("{}/api/register", self.base_url))
            .json(&req)
            .send()
            .await?;
        
        let api_response: ApiResponse<LoginResponse> = response.json().await?;
        
        if api_response.success {
            if let Some(login_data) = api_response.data {
                self.token = Some(login_data.token.clone());
                self.user_id = Some(login_data.user_id);
                Ok(login_data)
            } else {
                Err("No data in response".into())
            }
        } else {
            Err(api_response.message.into())
        }
    }
    
    pub async fn login(&mut self, username: &str, password: &str) -> Result<LoginResponse, Box<dyn std::error::Error + Send + Sync>> {
        let req = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };
        
        let response = self.client
            .post(&format!("{}/api/login", self.base_url))
            .json(&req)
            .send()
            .await?;
        
        let api_response: ApiResponse<LoginResponse> = response.json().await?;
        
        if api_response.success {
            if let Some(login_data) = api_response.data {
                self.token = Some(login_data.token.clone());
                self.user_id = Some(login_data.user_id);
                Ok(login_data)
            } else {
                Err("No data in response".into())
            }
        } else {
            Err(api_response.message.into())
        }
    }
    
    pub async fn submit_certificate(
        &self,
        certificate_data: &str,
        device_info: &str,
        sanitization_method: &str,
    ) -> Result<Certificate, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.token.as_ref().ok_or("Not logged in")?;
        
        let req = SubmitCertificateRequest {
            certificate_data: certificate_data.to_string(),
            device_info: device_info.to_string(),
            sanitization_method: sanitization_method.to_string(),
        };
        
        let response = self.client
            .post(&format!("{}/api/certificates", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&req)
            .send()
            .await?;
        
        let api_response: ApiResponse<Certificate> = response.json().await?;
        
        if api_response.success {
            api_response.data.ok_or("No certificate data in response".into())
        } else {
            Err(api_response.message.into())
        }
    }
    
    pub async fn get_certificates(&self, limit: i64, offset: i64) -> Result<CertificateResponse, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.token.as_ref().ok_or("Not logged in")?;
        
        let response = self.client
            .get(&format!("{}/api/certificates?limit={}&offset={}", self.base_url, limit, offset))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;
        
        let api_response: ApiResponse<CertificateResponse> = response.json().await?;
        
        if api_response.success {
            api_response.data.ok_or("No certificate data in response".into())
        } else {
            Err(api_response.message.into())
        }
    }
    
    pub async fn get_sanitization_logs(&self, limit: i64, offset: i64) -> Result<SanitizationLogResponse, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.token.as_ref().ok_or("Not logged in")?;
        
        let response = self.client
            .get(&format!("{}/api/logs?limit={}&offset={}", self.base_url, limit, offset))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;
        
        let api_response: ApiResponse<SanitizationLogResponse> = response.json().await?;
        
        if api_response.success {
            api_response.data.ok_or("No log data in response".into())
        } else {
            Err(api_response.message.into())
        }
    }
    
    pub fn is_logged_in(&self) -> bool {
        self.token.is_some() && self.user_id.is_some()
    }
    
    pub fn get_user_id(&self) -> Option<Uuid> {
        self.user_id
    }
    
    pub fn logout(&mut self) {
        self.token = None;
        self.user_id = None;
    }
}