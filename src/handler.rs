use std::env;

use line_bot_sdk::{
    models::{
        message::text::TextMessage,
        webhook_event::{Event, Message, Text},
    },
    Client,
};
use log::info;

use actix_web::{rt::spawn, HttpResponse, Responder};
use dotenv::dotenv;
use line_bot_sdk::extractor::CustomHeader;
use line_bot_sdk::models::message::MessageObject;
use line_bot_sdk::models::webhook_event;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

pub async fn handler(
    context: String,
    custom_header: CustomHeader,
) -> Result<impl Responder, AppError> {
    info!("Request body: {}", context);

    dotenv().ok();

    let client = Client::new(
        env::var("CHANNEL_ACCESS_TOKEN").map_err(AppError::Env)?,
        env::var("CHANNEL_SECRET").map_err(AppError::Env)?,
        env::var("CHANNE_ID").map_err(AppError::Env)?,
    );

    let signature = &custom_header.x_line_signature;
    client
        .verify_signature(signature, &context)
        .map_err(AppError::LineBotSdkError)?;

    let context: webhook_event::Root =
        serde_json::from_str(&context).map_err(AppError::SerdeJson)?;
    spawn(async { webhook_handler(context, client).await });
    Ok(HttpResponse::Ok().body(""))
}

async fn webhook_handler(
    context: webhook_event::Root,
    client: Client,
) -> Result<HttpResponse, AppError> {
    for event in &context.events {
        reply(event, &client).await?;
    }
    Ok(HttpResponse::Ok().json("Ok"))
}

async fn reply_message(client: &Client, reply_token: &str, messages: Vec<MessageObject>) {
    let reply_response = client.reply(reply_token, messages, None).await;
    if let Err(err) = reply_response {
        println!("エラーが発生しました: {}", err);
    }
}

fn get_text_message(event: &Event) -> Option<&Text> {
    match &event.message {
        Some(Message::Text(message)) => Some(message),
        _ => None,
    }
}

fn create_text_message(text: &str) -> MessageObject {
    MessageObject::Text(TextMessage::builder().text(text).build())
}

async fn reply(event: &Event, client: &Client) -> Result<(), AppError> {
    let reply_token = match event.reply_token.clone() {
        Some(reply_token) => reply_token,
        None => return Ok(()),
    };

    match get_text_message(event) {
        Some(text) => {
            let messages = vec![create_text_message(&text.text)];
            reply_message(client, &reply_token, messages).await;
        }
        None => {
            let messages = vec![create_text_message(
                "テキストメッセージ以外のイベントには対応していません",
            )];
            reply_message(client, &reply_token, messages).await;
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReplyMessage {
    #[serde(rename(serialize = "replyToken"))]
    reply_token: String,
    messages: Vec<MessageObject>,
}
