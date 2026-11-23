use std::sync::LazyLock;

use axum::{http::{header, StatusCode}, response::{IntoResponse, Response}};

struct SitemapRoute {
    loc: &'static str,
    lastmod: &'static str,
}

// route = "/sitemap.xml"
static SITEMAP_XML: LazyLock<String> = LazyLock::new(|| {
    let routes = [
        SitemapRoute { loc: "https://pablech-dev.com/",      lastmod: "2026-11-20" },
        SitemapRoute { loc: "https://pablech-dev.com/about", lastmod: "2026-10-24" },
        SitemapRoute { loc: "https://pablech-dev.com/github", lastmod: "2026-11-17" },
        SitemapRoute { loc: "https://pablech-dev.com/api/resume", lastmod: "2026-11-16" },
        SitemapRoute { loc: "https://pablech-dev.com/api/links", lastmod: "2026-11-16" },
    ];

    let mut xml = String::with_capacity(512);
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#);

    for route in routes {
        xml.push_str(&format!(
            r#"
    <url>
        <loc>{}</loc>
        <lastmod>{}</lastmod>
    </url>"#,
            route.loc, route.lastmod
        ));
    }

    xml.push_str("\n</urlset>");
    xml
});

pub async fn sitemap() -> Response {(
        StatusCode::OK, [(header::CONTENT_TYPE, "application/xml; charset=utf-8")], SITEMAP_XML.as_str(),
    ).into_response()
}