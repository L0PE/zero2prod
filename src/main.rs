use actix_web::{web, App, HttpRequest, HttpServer, Responder};

async fn heals_check(request: HttpRequest) -> impl Responder {
    let name = request.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", name)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(heals_check))
            .route("/{name}", web::get().to(heals_check))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
