use axum::{http::{header, StatusCode}, response::{IntoResponse, Response}};

// route = "/llms.txt"
pub async fn llms_txt_handler() -> Response {
    let body = "# Pablech-dev\n  \
        > Portfolio de desenvolvedor — Rust, Python, RL aplicado.\n\n\
        ## Sobre\n  \
        /about: Quem sou, o projeto BRISE (IA para automação de ar-condicionado com RL)\n\n\
        ## Projetos\n  \
        /github: Repositórios públicos no GitHub\n";
    
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body).into_response()
}