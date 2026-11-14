use axum::Json;

use crate::handlers::structs::LinkItem;

const LINKS: &[LinkItem] = &[
    LinkItem { label: "Github", mask: "@Pablech", url: "https://www.github.com/Pablech" },
    LinkItem { label: "Linkedin", mask: "in/Pablo-Echagüe", url: "https://www.linkedin.com/in/pablo-echag%C3%BCe-140318395/" },
];

// router = GET /api/links
pub async fn links() -> Json<Vec<LinkItem>> {
    Json(LINKS.into())
}