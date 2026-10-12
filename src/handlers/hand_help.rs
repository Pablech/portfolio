use axum::{http::header, Json};

use crate::handlers::models::BasicRes;

pub async fn help_handler() -> ([(header::HeaderName, &'static str); 1], Json<BasicRes>) {
    let help = "Comandos disponíveis:\n  \
        help       - Mostra esta mensagem\n  \
        github     - Lista os projetos públicos do GitHub\n  \
        email      - Me envie um e-mail\n  \
        theme + style \n  \
            \ttheme tty      - Altera para o tema TTY Retrô (Preto/Verde - Padrão)\n  \
            \ttheme white    - Altera para o tema Branco (Fundo branco / Letra preta)\n  \
            \ttheme tty-new  - Altera para o tema TTY New (Preto / Letra branca)\n  \
        about      - Quem sou eu e o que ando a construir\n  \
        clear      - Limpa o terminal";
    
    ([(header::CACHE_CONTROL, "no-store")], Json(BasicRes { output: help.to_string() }))
}