use std::net::TcpListener;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_web::dev::Server;

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health_check))
    })
        .listen(listener)?
        .run();

    Ok(server)
}
