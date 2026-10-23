use axum::{http::header, Json};

use crate::handlers::models::BasicRes;

pub async fn help_handler() -> ([(header::HeaderName, &'static str); 1], Json<BasicRes>) {
    let help = "Comandos disponíveis:\n  \
        help       - Mostra esta mensagem\n  \
        github     - Lista os projetos públicos do GitHub\n  \
        email      - Me envie um e-mail\n  \
        theme + style \n  \
            \ttheme tty\n  \
            \ttheme white\n  \
            \ttheme matrix\n  \
            \ttheme pink\n  \
        about      - Quem sou eu e o que ando a construir\n  \
        clear      - Limpa o terminal\n  \
        new        - Abre outro terminal\n  \
        exit       - Fecha o terminal\n  \
        keys       - Para veres os atalhos do teclado";
    
    ([(header::CACHE_CONTROL, "no-store")], Json(BasicRes { output: help.to_string() }))
}