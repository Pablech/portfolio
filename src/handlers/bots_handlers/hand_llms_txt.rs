use axum::{http::{header, StatusCode}, response::{IntoResponse, Response}};

// route = "/llms.txt"
pub async fn llms_txt_handler() -> Response {
    let body: &str = "# Pablech-dev\n\n\
        > Portfólio interativo de programador simulando um ambiente TTY (Terminal), desenvolvido em Rust (Axum) e JavaScript.\n\n\
        ## Secções Principais\n\
        - [Sobre](https://pablech-dev.com/about): Quem sou, background técnico e competências em Rust, Python e Aprendizagem por Reforço (RL).\n\
        - [GitHub / Projetos](https://pablech-dev.com/github): Lista de repositórios públicos, incluindo o código-fonte deste portfólio e projetos de automação (ex: BRISE).\n\
        ## Recursos Adicionais\n\
        - [Contacto por E-mail](https://pablech-dev.com/email): Funcionalidade de envio de mensagens integrada no terminal simulado.\n";

    (StatusCode::OK, [(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body).into_response()
}