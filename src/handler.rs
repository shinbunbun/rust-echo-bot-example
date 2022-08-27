use std::{
    env,
    sync::{Arc, Mutex},
};

use line_bot_sdk::{
    models::{
        message::text::TextMessage,
        webhook_event::{Event, Message, Root, Text},
    },
    Client,
};
use log::info;

use actix_web::{get, rt::spawn, web, HttpResponse, Responder};
use line_bot_sdk::extractor::CustomHeader;
use line_bot_sdk::models::message::MessageObject;
use line_bot_sdk::models::webhook_event;

use serde::{Deserialize, Serialize};

use crate::{error::AppError, AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReplyMessage {
    reply_token: String,
    messages: Vec<MessageObject>,
}

pub async fn handler(
    context: String,
    custom_header: CustomHeader,
    app_state: web::Data<AppState>,
) -> impl Responder {
    info!("Request body: {}", context);

    let client = Arc::clone(&app_state.line_client);

    let signature = get_signature_from_header(&custom_header);

    verify_signature(&client, signature, &context).unwrap();

    let webhook_event = get_webhook_event(&context).unwrap();

    spawn(async move { webhook_handler(&webhook_event, &client).await.unwrap() });

    HttpResponse::Ok().body("")
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
    Ok(serde_json::from_str(context)?)
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

    let messages = match get_text_message(event) {
        Some(text) => vec![create_text_message(&text.text)],
        None => vec![create_text_message(
            "テキストメッセージ以外のイベントには対応していません",
        )],
    };

    client
        .reply(&reply_token, messages, None)
        .await
        .map_err(AppError::LineBotSdkError)?;

    Ok(())
}
