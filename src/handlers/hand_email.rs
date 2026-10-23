use axum::{Json, extract::State};
use std::sync::Arc;
use lettre::{Message, SmtpTransport, Transport, message::Mailbox, Address};
use lettre::transport::smtp::authentication::Credentials;

use crate::handlers::models::{SenfEmailReq, BasicRes};
use crate::core::env_settings::EnvSetting;

// route = "/api/send-email"
pub async fn send_email_handler(State(env): State<Arc<EnvSetting>>,
        Json(payload): Json<SenfEmailReq>) -> Json<BasicRes> {
    
    let user_smtp = env.user_smtp.clone();
    let token_smtp = env.token_smtp.clone();
    let server_smtp = env.server_smtp.clone();

    let visitor_email = if payload.from.trim().is_empty() {
        "Anónimo".to_string()
    } else {
        payload.from.to_string()
    };

    let raw_subject = if payload.subject.trim().is_empty() {
        "Sem assunto".to_string()
    } else {
        payload.subject.to_string()
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
            return Json(BasicRes { 
                output: format!("<span style='color: #f7768e;'>[✗] Erro na estrutura da mensagem: {}</span>", e), 
            });
        }
    };

    let creds = Credentials::new(user_smtp, token_smtp);
    
    let mailer = match SmtpTransport::starttls_relay(&server_smtp) {
        Ok(builder) => builder.credentials(creds).build(),
        Err(e) => {
            return Json(BasicRes {
                output: format!("<span style='color: #f7768e;'>[✗] Erro de conexão SMTP: {}</span>", e),
            });
        }
    };

    let result = tokio::task::spawn_blocking(move || mailer.send(&email)).await;

    match result {
        Ok(Ok(_)) => Json(BasicRes {
            output: "<span style='color: #9ece6a;'>[✓] E-mail enviado com sucesso!</span>".to_string(),
        }),
        Ok(Err(e)) => Json(BasicRes {
            output: format!("<span style='color: #f7768e;'>[✗] Erro ao enviar: {}</span>", e),
        }),
        Err(_) => Json(BasicRes {
            output: "<span style='color: #f7768e;'>[✗] Erro interno ao processar a tarefa</span>".to_string(),
        }),
    }
}
