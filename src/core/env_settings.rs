use std::env;
use tracing::error;

#[derive(Debug, Clone)]
pub struct EnvSetting {
    pub git_token: String,
    pub user_agent: String,
    pub user_name: String,
    pub max_retries: u32,
    pub request_timeout: u64,
    pub url: String,
    pub tunner: String,
    pub cloudflared_cred: String,
    pub token_smtp: String,
    pub user_smtp: String,
    pub server_smtp: String,
    pub port_smtp: String,
}

impl EnvSetting {
    pub fn new() -> Self {
        let _ = dotenvy::dotenv();
        let mut missing_vars = Vec::new();

        let mut get_required = |key: &str| -> String {
            match env::var(key).map(|s| s.trim().to_string()) {
                Ok(val) if !val.is_empty() => val, _ => {
                    missing_vars.push(key.to_string());
                    String::new()
                }
            }
        };

        let git_token = get_required("GIT_TOKEN");
        let user_agent = get_required("USER_AGENT");
        let url = get_required("URL");
        let user_name = get_required("GIT_USER_NAME");
        let tunner = get_required("TUNNEL");
        let cloudflared_cred = get_required("CLOUDFLARED_CRED");
        let token_smtp = get_required("TOKEN_SMTP");
        let user_smtp = get_required("USER_SMTP");
        let server_smtp = get_required("SERVER_SMTP");
        let port_smtp = get_required("PORT_SMTP");

        if !missing_vars.is_empty() {
            error!("Variáveis de ambiente ausentes: {}", missing_vars.join(", ").to_uppercase());
            std::process::exit(1);
        }

        let max_retries = env::var("MAX_RETRIES")
            .unwrap_or_else(|_| "3".to_string()).parse::<u32>().unwrap_or(3);
        let request_timeout = env::var("REQUEST_TIMEOUT")
            .unwrap_or_else(|_| "10".to_string()).parse::<u64>().unwrap_or(10);

        Self { git_token, user_agent, user_name, max_retries, request_timeout, url, tunner, cloudflared_cred, token_smtp, user_smtp, server_smtp, port_smtp }
    }
}