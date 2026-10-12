use axum::http::HeaderValue;
use reqwest::{Method, header::{HeaderMap, AUTHORIZATION, USER_AGENT, ACCEPT}};
use serde_json::from_value;
use base64::{Engine as _, engine::general_purpose};
use tracing::warn;

use crate::repositories::httpc::httpc::HttpC;
use crate::core::env_settings::EnvSetting;
use crate::repositories::github::models::{
    Repo, RepoContent, RepoFile, RepoInfo};


pub struct GitHub {
    env: EnvSetting,
    httpc: HttpC,
    acept: String,
    auth_value: String,
    user_agent: String,
}

impl GitHub {
    pub fn new(env: &EnvSetting, httpc: &HttpC) -> Self {
        Self { 
            env: env.clone(),
            httpc: httpc.clone(),
            acept: "application/vnd.github+json".parse().unwrap(), 
            auth_value: format!("Bearer {}", &env.git_token), 
            user_agent: env.user_agent.clone(),
        }
    }

    pub async fn fetch_repos(&self, include_private: bool) 
            -> Result<Vec<Repo>, Box<dyn std::error::Error + Send + Sync + 'static>> {
                
        let url: String = format!("{}/user/repos", &self.env.url);
        
        let mut header = HeaderMap::new();
        header.insert(AUTHORIZATION, HeaderValue::from_str(&self.auth_value)?);
        header.insert(USER_AGENT, HeaderValue::from_str(&self.user_agent)?);

        let response = &self.httpc.request(
            Method::GET, 
            &url,
            Some(header), 
            None, 
            None,
        ).await?;

        let repos: Vec<Repo> = from_value(response.clone())?;

        let filtered_repos: Vec<Repo> = if include_private {
            repos.into_iter().collect()
        } else {
            repos.into_iter().filter(|r| !r.private).collect()
        };

        Ok(filtered_repos)
    }

    pub async fn fetch_readme(&self, full_name: &str) 
            -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        
        let url = format!("{}/repos/{}/readme", &self.env.url, full_name);

        let mut header = HeaderMap::new();
        header.insert(USER_AGENT, HeaderValue::from_str(&self.user_agent)?);
        header.insert(ACCEPT, HeaderValue::from_str(&self.acept)?);

        let response = &self.httpc.request(
            Method::GET, 
            &url, 
            Some(header), 
            None,
            None,
        ).await?.clone();

        if let Some(content_b64) = response["content"].as_str() {
            let clean_b64 = content_b64.replace("\n", "");
            let bytes = general_purpose::STANDARD.decode(clean_b64)?;
            let readme_text = String::from_utf8(bytes)?;
            return Ok(readme_text);
        }

        Ok("NO content".to_string())
    }

    pub async fn fetch_structure(&self, full_name: &str)
            -> Result<Vec<RepoContent>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        
        let url = format!("{}/repos/{}/contents/",&self.env.url, &full_name);

        let mut header = HeaderMap::new();
        header.insert(USER_AGENT, HeaderValue::from_str(&self.user_agent)?);
        header.insert(ACCEPT, HeaderValue::from_str(&self.acept)?);

        let response = &self.httpc.request(
            Method::GET, 
            &url, 
            Some(header), 
            None,
            None,
        ).await?;
        let contents: Vec<RepoContent> = from_value(response.clone())?;
        Ok(contents)
    }

    pub async fn get_github_json(&self) -> String {
        let repos = match self.fetch_repos(false).await {
            Ok(r) => r,
            Err(e) => {
                warn!("Erro ao buscar repos: {}", e);
                return "[]".to_string(); 
            }
        };

        let mut frontend_repos = Vec::new();

        for repo in repos {
            let desc = repo.description.clone().unwrap_or_else(|| "Sem descrição disponível.".to_string());
            let lang = repo.language.clone().unwrap_or_else(|| "N/A".to_string());
            
            let mut files = Vec::new();

            if let Ok(readme_text) = self.fetch_readme(&repo.full_name).await {
                files.push(RepoFile {
                    name: "README.md".to_string(),
                    code: readme_text,
                });
            } else {
                files.push(RepoFile {
                    name: "README.md".to_string(),
                    code: "// Sem README encontrado.".to_string(),
                });
            }

            if let Ok(structure) = self.fetch_structure(&repo.full_name).await {
                for item in structure {
                    if item.name.to_lowercase() != "readme.md" {
                        files.push(RepoFile {
                            name: item.name,
                            code: "// O código deste ficheiro não foi descarregado para poupar a API.\n// Abre o repositório original para veres o código!".to_string(),
                        });
                    }
                }
            }

            frontend_repos.push(RepoInfo {
                id: repo.id as u64,
                name: repo.name.clone(),
                desc,
                lang,
                url: repo.html_url.clone(),
                files,
            });
        }

        serde_json::to_string(&frontend_repos).unwrap_or_else(|_| "[]".to_string())
    }
}