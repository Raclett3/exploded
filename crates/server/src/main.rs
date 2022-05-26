mod game;

use actix::prelude::*;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use game::*;

#[get("/ws")]
async fn websocket(
    req: HttpRequest,
    stream: web::Payload,
    matchmaker: web::Data<Addr<Matchmaker>>,
) -> impl Responder {
    ws::start(Player::new(matchmaker.as_ref().clone()), &req, stream)
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!\n")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matchmaker = Matchmaker::new().start();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(matchmaker.clone()))
            .service(hello)
            .service(websocket)
    })
    .bind(("127.0.0.1", 9000))?
    .run()
    .await
}
