use crate::handlers::models::PortfolioTemplate;

// route = "/"
pub async fn home_handler() -> Result<PortfolioTemplate, String> {
    let timestamp = chrono::Utc::now().timestamp();
    let template = PortfolioTemplate {
        title: "Portfolio OS".to_string(),
        timestamp: timestamp,
    };
    Ok(template)
}