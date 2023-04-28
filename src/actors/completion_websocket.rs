use std::panic;

use actix::StreamHandler;
use actix_web::web;
use actix_web_actors::ws;
use actix::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{
    data::models::{self, Pool},
    operators::message_operator::{
        get_messages_for_topic_query
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
    InvalidMessage(&'static str),
}

enum Response {
    Messages(Vec<models::Message>),
    Pong,
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
                    return Command::InvalidMessage("Invalid message");
                }
                let message = parsed_message.unwrap();
                match message.command.as_str() {
                    "ping" => Command::Ping,
                    "prompt" => {
                        Command::Prompt(message.previous_messages.expect("Previous messages not found"))
                    }
                    "regenerateMessage" => Command::RegenerateMessage,
                    "changeTopic" => {
                        Command::ChangeTopic(message.topic_id.expect("Topic id not found"))
                    }
                    "stop" => Command::Stop,
                    _ => Command::InvalidMessage("Invalid command {:?}", ),
                }
            }
            ws::Message::Binary(_) =>Command::InvalidMessage("Binary not a valid operation"),
            ws::Message::Close(_) => Command::InvalidMessage("Close not a operation"),
            ws::Message::Continuation(_) => Command::InvalidMessage("Continuation not a operation"),
            ws::Message::Nop => Command::InvalidMessage("Nop not a operation"),
            ws::Message::Pong(_) => Command::InvalidMessage("Pong not a operation"),
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
            Err(_) => Command::InvalidMessage("Invalid message"),
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
                let messages = get_messages_for_topic_query(topic_id, &self.pool);
                match &messages {
                    Ok(messages) => {
                        ctx.text(serde_json::to_string(messages).unwrap())
                    }
                    Err(err) => {
                        ctx.text(serde_json::to_string(err).unwrap())
                    }
                }
                // ctx.text(serde_json::to_string(&messages).unwrap());
            }
            Command::Stop => {
                log::info!("Stop received");
                todo!();
            }
            Command::InvalidMessage(_) => {
                log::info!("Invalid message received");
                todo!();
            }
        }
    }
}
