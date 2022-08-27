mod error;
mod handler;
mod router;

use std::sync::Arc;

use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use line_bot_sdk::Client;

pub struct AppState {
    line_client: Arc<Client>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    read_dotenv();

    env_logger::init();
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(router::router)
            .app_data(web::Data::new(AppState {
                line_client: Arc::new(Client::new(
                    std::env::var("CHANNEL_ACCESS_TOKEN").unwrap(),
                    std::env::var("CHANNEL_SECRET").unwrap(),
                    std::env::var("CHANNEL_ID").unwrap(),
                )),
            }))
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}

fn read_dotenv() {
    dotenv().ok();
}
