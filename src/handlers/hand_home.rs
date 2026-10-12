use crate::handlers::models::PortfolioTemplate;

// route = "/"
pub async fn home_handler() -> Result<PortfolioTemplate, String> {
    Ok(PortfolioTemplate { title: "Pablo Echagüe: ~/portfolio".to_string() })
}