use axum::{http::{header, StatusCode}, response::{IntoResponse, Response}};

// route = "/llms.txt"
pub async fn llms_txt() -> Response {
    let body: &str = "# Pablech-dev\n\n\
        > Portfólio interativo de programador simulando um ambiente TTY (Terminal), desenvolvido em Rust (Axum) e JavaScript.\n\n\
        ## Secções Principais\n\
        - [Sobre](https://pablech-dev.com/api/about): Quem sou, background técnico e competências em Rust, Python e Aprendizagem por Reforço (RL).\n\
        - [GitHub / Projetos](https://pablech-dev.com/api/github): Lista de repositórios públicos, incluindo o código-fonte deste portfólio e projetos de automação (ex: BRISE).\n\
        - [Currículo (PDF)](https://pablech-dev.com/api/resume): Currículo completo em PDF — formação (Tecnólogo ADS/UNISINOS), certificações Cisco Python, experiência em Rust/Python/RL.\n\
        - [Ajuda por contexto](https://pablech-dev.com/api/help/os): Documentação dos comandos do terminal por módulo (kinds: `os`, `games`).\n\n\
        ## Perfil\n\
        - GitHub: https://github.com/Pablech \n\
        - LinkedIn: https://www.linkedin.com/in/pablo-echag%C3%BCe-140318395/ \n\n\
        ## Jogos Multiplayer (API WebSocket)\n\
        - [Criar sala](https://pablech-dev.com/games/rooms): `POST /games/rooms` (flags via terminal: jogo, nome, figura, modo pvp/cpu). Retorna `room_id` e tokens de join/play.\n\
        - Entrar em sala: conectar via `wss://pablech-dev.com/games/ws/{room_id}/join` com o token de join recebido, nome, figura.\n\
        - Fluxo completo no terminal: `create`, `join`, `quit`, `rematch`, `reconnect` — veja `/api/help/games`.\n\
        - Observação: os jogos (ex: Connect 4) são partidas em tempo real via WebSocket; não há página HTML navegável, apenas o protocolo acima.\n\n\
        ## Recursos Adicionais\n\
        - Contacto: mailto:contato@pablech-dev.com\n";

    (StatusCode::OK, [(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body).into_response()
}