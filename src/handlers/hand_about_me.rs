use axum::{http::header, Json};

use crate::handlers::models::BasicRes;

// route = "/api/about"
pub async fn about_handler() -> ([(header::HeaderName, &'static str); 1], Json<BasicRes>) {
    let about = "Sobre mim:\n  \
        Sou estudante universitário e desenvolvedor full-stack apaixonado por\n  \
        sistemas, automação e código de baixo nível e com vontade de aprender.\n\n  \
        Principais interesses:\n  \
        Rust          - Backends performáticos, CLIs e sistemas confiáveis\n  \
        Python        - Machine Learning e automação\n  \
        Linux (Arch)  - Diário, com tudo customizado na mão\n  \
        IA/RL         - Agentes inteligentes aplicados a problemas reais\n\n  \
        Projeto principal - BRISE (Residência TIC 55 Brisa-Unisinos):\n  \
        Sistema de automação inteligente de ar-condicionados splits com\n  \
        Inteligência Artificial. Treino agentes de reinforcement learning\n  \
        (PPO) em simulação com Sinergym/EnergyPlus, que decidem setpoint e\n  \
        modo de operação priorizando: conforto térmico > consumo de energia\n  \
        > umidade e qualidade do ar.\n  \
        Stack: Python, TensorFlow/Keras, Gymnasium, PostgreSQL, Docker e\n  \
        Apache Arrow Flight para ingestão de dados de sensores reais.\n\n  \
        Este portfólio:\n  \
        Desenvolvido do zero em Rust com Axum — do servidor à interface\n  \
        que estás a ver. Terminal retrô, temas alternáveis e zero\n  \
        frameworks de frontend. Só código.\n  \
        Ainda em desenvolvimento...\n\n  \
        Filosofia:\n  \
        Acredito que boa engenharia é fazer mais com menos — código enxuto\n  \
        e sistemas compreensíveis. Democratizar tecnologia é escrever\n  \
        software com performance e baixo custo computacional: se um\n  \
        programa roda bem em qualquer máquina, ele serve a qualquer\n  \
        pessoa, não só a quem tem hardware caro.\n\n  \
        Contato: digita 'email' para me enviares uma mensagem.";

    ([(header::CACHE_CONTROL, "no-store")], Json(BasicRes { output: about.to_string() }))
}