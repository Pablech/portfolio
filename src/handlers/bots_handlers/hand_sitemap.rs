use axum::{http::
    {header, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::Utc;

struct SitemapRoute {
    loc: &'static str,
    changefreq: &'static str,
    priority: f32,
}

// route =  "/sitemap.xml"
pub async fn sitemap_handler() -> Response {
    let routes = vec![
        SitemapRoute { loc: "https://pablech-dev.com/", changefreq: "weekly", priority: 1.0 },
        SitemapRoute { loc: "https://pablech-dev.com/about", changefreq: "monthly", priority: 0.8 },
        SitemapRoute { loc: "https://pablech-dev.com/github", changefreq: "daily", priority: 0.9 },
    ];

    let current_date = Utc::now().format("%Y-%m-%d"); 

    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
        <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#);

    for route in routes {
        xml.push_str(&format!(
            r#"
                <url>
                    <loc>{}</loc>
                    <lastmod>{}</lastmod>
                    <changefreq>{}</changefreq>
                    <priority>{:.1}</priority>
                </url>"#,
            route.loc, current_date, route.changefreq, route.priority
        ));
    }

    xml.push_str("\n</urlset>");

    (StatusCode::OK, [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        xml, ).into_response()
}