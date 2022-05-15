use actix::prelude::*;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;

struct Server {
    sessions: Vec<Addr<Session>>,
}

impl Server {
    fn new() -> Self {
        Server {
            sessions: Vec::new(),
        }
    }
}

impl Actor for Server {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
struct Connect(Addr<Session>);

impl Handler<Connect> for Server {
    type Result = ();

    fn handle(&mut self, Connect(addr): Connect, _ctx: &mut Self::Context) {
        self.sessions.push(addr);
    }
}

struct Session {
    server_addr: Addr<Server>,
}

impl Session {
    fn new(server_addr: Addr<Server>) -> Self {
        Session { server_addr }
    }
}

impl Actor for Session {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.text("Connection established");
        self.server_addr.do_send(Connect(ctx.address()));
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Session {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                let text_slice: &[u8] = text.as_ref();
                let text = String::from_utf8(text_slice.to_vec()).unwrap();
                self.server_addr.do_send(Text(text));
            }
            _ => (),
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct Text(String);

impl Handler<Text> for Session {
    type Result = ();

    fn handle(&mut self, Text(msg): Text, ctx: &mut Self::Context) {
        ctx.text(msg);
    }
}

impl Handler<Text> for Server {
    type Result = ();

    fn handle(&mut self, Text(msg): Text, _ctx: &mut Self::Context) {
        for session in &self.sessions {
            session.do_send(Text(msg.clone()));
        }
    }
}

#[get("/ws")]
async fn websocket(
    req: HttpRequest,
    stream: web::Payload,
    server_addr: web::Data<Addr<Server>>,
) -> impl Responder {
    ws::start(Session::new(server_addr.as_ref().clone()), &req, stream)
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!\n")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server_addr = Server::new().start();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server_addr.clone()))
            .service(hello)
            .service(websocket)
    })
    .bind(("127.0.0.1", 9000))?
    .run()
    .await
}
