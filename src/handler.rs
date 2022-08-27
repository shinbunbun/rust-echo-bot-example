use std::env;

use line_bot_sdk::{
    models::{
        message::text::TextMessage,
        webhook_event::{Event, Message, Root, Text},
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReplyMessage {
    #[serde(rename(serialize = "replyToken"))]
    reply_token: String,
    messages: Vec<MessageObject>,
}

pub async fn handler(
    context: String,
    custom_header: CustomHeader,
) -> Result<impl Responder, AppError> {
    info!("Request body: {}", context);

    read_dotenv();

    let client = create_line_bot_client()?;

    let signature = get_signature_from_header(&custom_header);

    verify_signature(&client, signature, &context)?;

    let webhook_event = get_webhook_event(&context)?;

    spawn(async move { webhook_handler(&webhook_event, &client).await });

    Ok(HttpResponse::Ok().body(""))
}

fn read_dotenv() {
    dotenv().ok();
}

fn create_line_bot_client() -> Result<Client, AppError> {
    Ok(Client::new(
        env::var("CHANNEL_ACCESS_TOKEN").map_err(AppError::Env)?,
        env::var("CHANNEL_SECRET").map_err(AppError::Env)?,
        env::var("CHANNE_ID").map_err(AppError::Env)?,
    ))
}

fn get_signature_from_header(custom_header: &CustomHeader) -> &str {
    &custom_header.x_line_signature
}

fn verify_signature(client: &Client, signature: &str, context: &str) -> Result<(), AppError> {
    client
        .verify_signature(signature, context)
        .map_err(AppError::LineBotSdkError)
}

fn get_webhook_event(context: &str) -> Result<Root, AppError> {
    serde_json::from_str(context).map_err(AppError::SerdeJson)
}

async fn webhook_handler(
    context: &webhook_event::Root,
    client: &Client,
) -> Result<HttpResponse, AppError> {
    for event in &context.events {
        reply(event, client).await?;
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
