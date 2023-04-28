use actix::StreamHandler;
use actix_web::web;
use actix_web_actors::ws;
use actix::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{
    data::models::{self, Pool},
    operators::message_operator::{
        get_messages_for_topic_query, user_owns_topic_query
    },
};

#[derive(Serialize, Deserialize, Debug)]
struct MessageDTO {
    command: String,
    previous_messages: Option<Vec<models::Message>>,
    topic_id: Option<uuid::Uuid>,
}

#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Ping,
    Prompt(Vec<models::Message>),
    RegenerateMessage,
    ChangeTopic(uuid::Uuid),
    Stop,
    InvalidMessage(String),
}

#[derive(Serialize, Deserialize, Debug)]
enum Response {
    Messages(Vec<models::Message>),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct CompletionWebSeocket {
    pub user_id: uuid::Uuid,
    pub topic_id: Option<uuid::Uuid>,
    pub last_pong: chrono::DateTime<chrono::Utc>,
    pub pool: web::Data<Pool>,
}

impl From<ws::Message> for Command {
    fn from(message: ws::Message) -> Self {
        match message {
            ws::Message::Ping(_msg) => {
                Command::Ping
            }
            ws::Message::Text(text) => {
                let parsed_message: Result<MessageDTO, serde_json::Error> = serde_json::from_str(&text);
                if parsed_message.is_err() {
                    return Command::InvalidMessage("Invalid message".to_string());
                }
                let message = parsed_message.unwrap();
                match (&message, message.command.as_str()) {
                    (_, "ping") => Command::Ping,
                    (msg, "prompt") if msg.previous_messages.is_some() => {
                        Command::Prompt(message.previous_messages.unwrap())
                    },
                    (_, "regenerateMessage") => Command::RegenerateMessage,
                    (msg, "changeTopic") if msg.topic_id.is_some() => {
                        Command::ChangeTopic(message.topic_id.unwrap())
                    },
                    (_, "stop") => Command::Stop,
                    (_, _) => Command::InvalidMessage("Missing properties".to_string()),
                }
            }
            ws::Message::Binary(_) =>Command::InvalidMessage("Binary not a valid operation".to_string()),
            ws::Message::Close(_) => Command::InvalidMessage("Close not a operation".to_string()),
            ws::Message::Continuation(_) => Command::InvalidMessage("Continuation not a operation".to_string()),
            ws::Message::Nop => Command::InvalidMessage("Nop not a operation".to_string()),
            ws::Message::Pong(_) => Command::InvalidMessage("Pong not a operation".to_string()),
        }
    }
}


impl Actor for CompletionWebSeocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(std::time::Duration::from_secs(1), |act, ctx| {
            if chrono::Utc::now().signed_duration_since(act.last_pong).num_seconds() > 10 {
                ctx.stop();
            }
        });
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for CompletionWebSeocket {

    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let command: Command = match msg {
            Ok(message) => message.into(),
            Err(_) => Command::InvalidMessage("Invalid message".to_string()),
        };
        match command {
            Command::Ping => {
                log::info!("Ping received");
                self.last_pong = chrono::Utc::now();
                ctx.pong("Pong".as_bytes());
            }
            Command::Prompt(_) => {
                log::info!("Prompt received");
                todo!();
            }
            Command::RegenerateMessage => {
                log::info!("Regenerate message received");
                todo!();
            }
            Command::ChangeTopic(topic_id) => {
                log::info!("Change topic received");
                if !user_owns_topic_query(self.user_id, topic_id, &self.pool) {
                    return ctx.text(serde_json::to_string(&Response::Error("User does not own topic".to_string())).unwrap());
                }
                let messages = get_messages_for_topic_query(topic_id, &self.pool);
                match &messages {
                    Ok(messages) => {
                        ctx.text(serde_json::to_string(&Response::Messages(messages.to_vec())).unwrap())
                    }
                    Err(err) => {
                        ctx.text(serde_json::to_string(err).unwrap())
                    }
                }
            }
            Command::Stop => {
                log::info!("Stop received");
                todo!();
            }
            Command::InvalidMessage(e) => {
                ctx.text(serde_json::to_string(&Response::Error(e.to_string())).unwrap())
            }
        }
    }
}
