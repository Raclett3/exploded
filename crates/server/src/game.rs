use actix::prelude::*;
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;

#[derive(Deserialize)]
#[serde(tag = "type")]
enum IncomingMessage {
    Join,
    Leave,
}

pub struct Matchmaker {
    waiting_players: HashSet<Addr<Player>>,
}

impl Matchmaker {
    pub fn new() -> Self {
        Matchmaker {
            waiting_players: HashSet::new(),
        }
    }
}

impl Actor for Matchmaker {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(5), |matchmaker, _ctx| {
            let pairs: Vec<_> = matchmaker
                .waiting_players
                .iter()
                .scan::<Option<&Addr<Player>>, _, _>(None, |acc, x| {
                    if let Some(left) = acc.take() {
                        Some(Some((left.clone(), x.clone())))
                    } else {
                        *acc = Some(x);
                        Some(None)
                    }
                })
                .flatten()
                .collect();
            for (left, right) in pairs {
                left.do_send(OutgoingMessage::Ready);
                right.do_send(OutgoingMessage::Ready);
                matchmaker.waiting_players.remove(&left);
                matchmaker.waiting_players.remove(&right);
            }
            println!("{:?}", matchmaker.waiting_players);
        });
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct Join(Addr<Player>);

impl Handler<Join> for Matchmaker {
    type Result = ();

    fn handle(&mut self, Join(player): Join, _ctx: &mut Self::Context) {
        self.waiting_players.insert(player);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct Leave(Addr<Player>);

impl Handler<Leave> for Matchmaker {
    type Result = ();

    fn handle(&mut self, Leave(player): Leave, _ctx: &mut Self::Context) {
        self.waiting_players.remove(&player);
    }
}

pub struct Player {
    matchmaker: Addr<Matchmaker>,
}

impl Player {
    pub fn new(matchmaker: Addr<Matchmaker>) -> Self {
        Player { matchmaker }
    }
}

impl Actor for Player {
    type Context = ws::WebsocketContext<Self>;

    fn stopped(&mut self, ctx: &mut Self::Context) {
        self.matchmaker.do_send(Leave(ctx.address()));
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Player {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                let text_slice: &[u8] = text.as_ref();
                let json = String::from_utf8(text_slice.to_vec()).unwrap();
                let msg = if let Ok(msg) = serde_json::from_str::<IncomingMessage>(&json) {
                    msg
                } else {
                    return;
                };
                match msg {
                    IncomingMessage::Join => self.matchmaker.do_send(Join(ctx.address())),
                    IncomingMessage::Leave => self.matchmaker.do_send(Leave(ctx.address())),
                }
            }
            _ => (),
        }
    }
}

#[derive(Serialize, Message)]
#[serde(tag = "type")]
#[rtype(result = "()")]
enum OutgoingMessage {
    Ready,
}

impl Handler<OutgoingMessage> for Player {
    type Result = ();

    fn handle(&mut self, msg: OutgoingMessage, ctx: &mut Self::Context) {
        if let Ok(msg) = serde_json::to_string(&msg) {
            ctx.text(msg);
        }
    }
}
