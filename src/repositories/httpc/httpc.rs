use std::collections::HashMap;
use std::time::Duration;
use reqwest::{Method, header::HeaderMap};
use tracing::warn;

use crate::core::env_settings::EnvSetting;


#[derive(Clone)]
pub struct HttpC {
    max_retries: u32,
    client: Option<reqwest::Client>,
}

impl HttpC {
    pub fn new(env: &EnvSetting) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(&env.user_agent)
            .timeout(Duration::from_secs(env.request_timeout))
            .pool_max_idle_per_host(100)
            .pool_idle_timeout(Duration::from_secs_f32(4.0))
            .build().expect("Falha ao inicializar cliente reqwest");

        Self { 
            max_retries: env.max_retries, 
            client: Some(client) 
        }
    }

    pub async fn close(&mut self) {
        self.client = None;
    }

    pub fn ready(&self) -> bool {
        self.client.is_some()
    }

    pub async fn request(&self, method: Method, url: &str, headers: Option<HeaderMap>,
            json_body: Option<&serde_json::Value>, query: Option<&HashMap<String, String>>) 
                    -> Result<serde_json::Value, Box<dyn  std::error::Error + Send + Sync>> {
                        
        let client = self.client.as_ref().ok_or("reqwest não inicializado")?;
        let mut last_error: Box<dyn std::error::Error + Send + Sync> = "Falha na requicição".into();

        for attempt in 1..=self.max_retries {
            let mut req_builder = client.request(method.clone(), url);

            if let Some(h) = headers.clone() {
                req_builder = req_builder.headers(h);
            }

            if let Some(q) = query {
                req_builder = req_builder.query(&q);
            }

            if let Some(j) = json_body {
                req_builder = req_builder.json(j);
            }

            match req_builder.send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.as_u16() == 404 {
                        warn!("Error: {}", status.as_u16());
                    }

                    if let Err(err) = response.error_for_status_ref() {
                        let error_text = response.text().await.unwrap_or_default();
                        warn!("Http {} em {} {} - Body: {}", status.as_u16(), method, url, error_text);

                        if status.is_client_error() {
                            return Err(Box::new(err));
                        }
                        last_error = Box::new(err);
                        continue;
                    }

                    if response.content_length() == Some(0) {
                        return  Ok(serde_json::json!({}));
                    }

                    match response.json::<serde_json::Value>().await {
                        Ok(json) => return Ok(json),
                        Err(e) => {
                            last_error = Box::new(e);
                            warn!("Tentativa {}/{} falhou ao parsear Json: {}", attempt, self.max_retries, last_error);
                        }
                    }
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    warn!("Tentativa {}/{} falhou: {} {} {}", attempt, self.max_retries, method, url, error_msg);
                    last_error = Box::new(e);
                }
            }
            let sleep_duration = Duration::from_secs_f32(0.5 * attempt as f32);
            tokio::time::sleep(sleep_duration).await;
        }
        Err(last_error)
    }
}
