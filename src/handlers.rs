use std::sync::Arc;
use askama::Template;
use axum::{extract::State, Json};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport, message::Mailbox, Address};
use serde::{Deserialize, Serialize};
// use axum::extract::Request;

use crate::EnvSetting;
use crate::repositories::github::github::GitHub;
use crate::repositories::outputs::outputs::{help};


#[derive(Template)]
#[template(path = "index.html")]
pub struct PortfolioTemplate {
    pub title: String,
}

#[derive(Deserialize)]
pub struct CommandReq {
    pub cmd: String,
}

#[derive(Serialize)]
pub struct CommandRes {
    pub output: String,
}

#[derive(Deserialize)]
pub struct SendEmailReq {
    pub from: String,
    pub subject: String,
    pub body: String,
}

// pub async fn debug_headers(req: Request) -> String {
//     let headers: Vec<_> = req
//         .headers()
//         .iter()
//         .filter(|(k, _)| k.as_str().contains("for") || k.as_str().contains("connecting"))
//         .map(|(k, v)| format!("{}: {}", k.as_str(), v.to_str().unwrap_or("invalid")))
//         .collect();
//     headers.join("\n")
// }

pub async fn home_handler() -> Result<PortfolioTemplate, String> {
    Ok(PortfolioTemplate { title: "Pablo Echagüe: ~/portfolio".to_string() })
}

pub async fn github_handler(State(gh): State<Arc<GitHub>>) -> Json<CommandRes> {
    let output = gh.get_github_json().await;

    Json(CommandRes { output })
}

pub async fn command_handler(Json(payload): Json<CommandReq>,) -> Json<CommandRes> {
    
    let cmd = payload.cmd.trim();
    
    let output = match cmd {
        "help" => help(),
        _ => format!("Comando não encontrado: <{}>. Digita 'help' para ajuda.", cmd),
    };

    Json(CommandRes { output })
}

pub async fn send_email_handler(State(env): State<Arc<EnvSetting>>,
        Json(payload): Json<SendEmailReq>,) -> Json<CommandRes> {

    let user_smtp = env.user_smtp.clone();
    let token_smtp = env.token_smtp.clone();
    let server_smtp = env.server_smtp.clone();

    let visitor_email = if payload.from.trim().is_empty() {
        "Anónimo".to_string()
    } else {
        payload.from.trim().to_string()
    };

    let raw_subject = if payload.subject.trim().is_empty() {
        "Sem assunto".to_string()
    } else {
        payload.subject.trim().to_string()
    };

    let subject = format!("[Portfolio CLI] {}", raw_subject);

    let body_content = format!(
        "Mensagem recebida via Portfolio CLI\n\
         ----------------------------------\n\
         Remetente: {}\n\
         Assunto: {}\n\n\
         Mensagem:\n{}\n",
        visitor_email, raw_subject, payload.body
    );

    let mut email_builder = Message::builder()
        .from(user_smtp.parse().unwrap())
        .to(user_smtp.parse().unwrap())
        .subject(subject);

    if let Ok(addr) = visitor_email.parse::<Address>() {
        let replay_mailbox = Mailbox::new(None, addr);
        email_builder = email_builder.reply_to(replay_mailbox);
    }

    let email = match email_builder.date_now().message_id(None).body(body_content) {
        Ok(m) => m,
        Err(e) => {
            return Json(CommandRes {
                output: format!("<span style='color: #f7768e;'>[✗] Erro na estrutura da mensagem: {}</span>", e),
            });
        }
    };

    let creds = Credentials::new(user_smtp, token_smtp);
    
    let mailer = match SmtpTransport::starttls_relay(&server_smtp) {
        Ok(builder) => builder.credentials(creds).build(),
        Err(e) => {
            return Json(CommandRes {
                output: format!("<span style='color: #f7768e;'>[✗] Erro de conexão SMTP: {}</span>", e),
            });
        }
    };

    let result = tokio::task::spawn_blocking(move || mailer.send(&email)).await;

    match result {
        Ok(Ok(_)) => Json(CommandRes {
            output: "<span style='color: #9ece6a;'>[✓] E-mail enviado com sucesso!</span>".to_string(),
        }),
        Ok(Err(e)) => Json(CommandRes {
            output: format!("<span style='color: #f7768e;'>[✗] Erro ao enviar: {}</span>", e),
        }),
        Err(_) => Json(CommandRes {
            output: "<span style='color: #f7768e;'>[✗] Erro interno ao processar a tarefa</span>".to_string(),
        }),
    }
}
