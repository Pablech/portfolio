pub fn help() -> String {
    let prompt = "Comandos disponíveis:\n  \
        help       - Mostra esta mensagem\n  \
        github     - Lista os projetos públicos do GitHub\n  \
        email      - Me envie um e-mail\n  \
        clear      - Limpa o terminal";
    prompt.to_string()
}