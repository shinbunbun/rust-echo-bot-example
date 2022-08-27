mod error;
mod handler;
mod router;

use actix_web::{middleware::Logger, App, HttpServer};
use dotenv::dotenv;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    read_dotenv();

    env_logger::init();
    HttpServer::new(move || App::new().wrap(Logger::default()).configure(router::router))
        .bind(("localhost", 8080))?
        .run()
        .await
}

fn read_dotenv() {
    dotenv().ok();
}
