use crate::routes::{health_check, subscribe};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn run(listener: TcpListener, database_pool: PgPool) -> Result<Server, std::io::Error> {
    let database_connection = web::Data::new(database_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health", web::get().to(health_check))
            .route("/subscribe", web::post().to(subscribe))
            .app_data(database_connection.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
