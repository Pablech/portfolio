use axum::Json;

use crate::handlers::models::BasicRes;

// route = "/api/help"
pub async fn help_handler() -> Json<BasicRes> {
    let help = "Comandos disponíveis:\n  \
        help       - Mostra esta mensagem\n  \
        github     - Lista os projetos públicos do GitHub\n  \
        email      - Me envie um e-mail\n  \
        clear      - Limpa o terminal";
    
    Json(BasicRes { output: help.to_string() })
}