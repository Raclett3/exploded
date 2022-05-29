use actix::prelude::*;
use actix_web_actors::ws;
use common::board::{Board, CellType};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

const WIDTH: usize = 8;
const HEIGHT: usize = 9;

fn cumulate(iter: impl Iterator<Item = usize>) -> impl Iterator<Item = usize> {
    iter.scan(0, |acc, x| {
        *acc += x;
        Some(*acc)
    })
}

#[derive(Clone)]
struct BombGenerator {
    rng: ThreadRng,
    generated: [usize; WIDTH],
}

impl BombGenerator {
    fn new() -> Self {
        let rng = thread_rng();
        BombGenerator {
            rng,
            generated: [0; WIDTH],
        }
    }

    fn possibility(&self) -> Vec<usize> {
        let max = self.generated.iter().copied().max().unwrap();
        self.generated
            .iter()
            .map(|x| 1 << ((max - x) * 2))
            .collect()
    }

    fn next_double(&mut self) -> (usize, usize) {
        let mut possibility = self.possibility();

        let mut sum = possibility.iter().sum::<usize>();
        let r = self.rng.gen_range(0..sum);

        let left = cumulate(possibility.iter().copied())
            .enumerate()
            .filter(|&(_, x)| r < x)
            .map(|(i, _)| i)
            .next()
            .unwrap();

        sum -= possibility[left];
        possibility[left] = 0;
        let r = self.rng.gen_range(0..sum);

        let right = cumulate(possibility.iter().copied())
            .enumerate()
            .filter(|&(_, x)| r < x)
            .map(|(i, _)| i)
            .next()
            .unwrap();

        self.generated[left] += 1;
        self.generated[right] += 1;
        (left, right)
    }

    fn next_single(&mut self) -> usize {
        let possibility = self.possibility();

        let sum = possibility.iter().sum::<usize>();
        let r = self.rng.gen_range(0..sum);

        let bomb = cumulate(possibility.iter().copied())
            .enumerate()
            .filter(|&(_, x)| r < x)
            .map(|(i, _)| i)
            .next()
            .unwrap();

        self.generated[bomb] += 1;
        bomb
    }
}

struct BoardManager {
    board: Board<WIDTH, HEIGHT>,
    generator: BombGenerator,
}

impl BoardManager {
    fn new() -> Self {
        BoardManager {
            board: Board::new(),
            generator: BombGenerator::new(),
        }
    }

    fn remove(&mut self, x: usize, y: usize) -> usize {
        let removed_cells = self.board.remove(x, y).len();
        self.board.apply_gravity();
        removed_cells
    }

    fn feed(&mut self, single: bool) -> [CellType; WIDTH] {
        let row = if single {
            let bomb = self.generator.next_single();
            let mut row = [CellType::Tile; WIDTH];
            row[bomb] = CellType::Bomb;
            row
        } else {
            let bombs = self.generator.next_double();
            let mut row = [CellType::Tile; WIDTH];
            row[bombs.0] = CellType::Bomb;
            row[bombs.1] = CellType::Bomb;
            row
        };
        self.board.feed(&row);
        row
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum IncomingMessage {
    Join,
    Leave,
    Remove { x: usize, y: usize },
}

struct Game {
    participants: HashMap<Addr<Player>, BoardManager>,
}

impl Game {
    fn new(participants: Vec<Addr<Player>>) -> Self {
        let participants = participants
            .into_iter()
            .map(|x| (x, BoardManager::new()))
            .collect();
        Game { participants }
    }
}

impl Actor for Game {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
struct Remove {
    player: Addr<Player>,
    x: usize,
    y: usize,
}

impl Handler<Remove> for Game {
    type Result = ();

    fn handle(&mut self, msg: Remove, _ctx: &mut Self::Context) {
        let Remove { player, x, y } = msg;
        let board = if let Some(board) = self.participants.get_mut(&player) {
            board
        } else {
            return;
        };
        let removed_cells = board.remove(x, y);
        if removed_cells > 0 {
            player.do_send(OutgoingMessage::Remove { x, y });
            let row = board.feed(false).map(|x| x == CellType::Bomb);
            player.do_send(OutgoingMessage::Feed { row });
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct Feed(Addr<Player>);

impl Handler<Feed> for Game {
    type Result = ();

    fn handle(&mut self, msg: Feed, _ctx: &mut Self::Context) {
        let Feed(player) = msg;
        if let Some(board) = self.participants.get_mut(&player) {
            let row = board.feed(false).map(|x| x == CellType::Bomb);
            player.do_send(OutgoingMessage::Feed { row });
        }
    }
}

pub struct Matchmaker {
    waiting_players: HashSet<Addr<Player>>,
    games: Vec<Addr<Game>>,
}

impl Matchmaker {
    pub fn new() -> Self {
        Matchmaker {
            waiting_players: HashSet::new(),
            games: Vec::new(),
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
                matchmaker.waiting_players.remove(&left);
                matchmaker.waiting_players.remove(&right);
                let game = Game::new(vec![left.clone(), right.clone()]).start();
                left.do_send(JoinGame(game.clone()));
                right.do_send(JoinGame(game.clone()));
                game.do_send(Feed(left));
                game.do_send(Feed(right));
                matchmaker.games.push(game);
            }
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
    game: Option<Addr<Game>>,
    matchmaker: Addr<Matchmaker>,
}

impl Player {
    pub fn new(matchmaker: Addr<Matchmaker>) -> Self {
        Player {
            matchmaker,
            game: None,
        }
    }
}

impl Actor for Player {
    type Context = ws::WebsocketContext<Self>;

    fn stopped(&mut self, ctx: &mut Self::Context) {
        self.matchmaker.do_send(Leave(ctx.address()));
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct JoinGame(Addr<Game>);

impl Handler<JoinGame> for Player {
    type Result = ();

    fn handle(&mut self, msg: JoinGame, ctx: &mut Self::Context) {
        let JoinGame(game) = msg;
        if let Ok(json) = serde_json::to_string(&OutgoingMessage::Ready) {
            ctx.text(json);
        }

        self.game = Some(game);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct LeaveGame;

impl Handler<LeaveGame> for Player {
    type Result = ();

    fn handle(&mut self, _msg: LeaveGame, _ctx: &mut Self::Context) {
        self.game = None;
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
                    IncomingMessage::Remove { x, y } => {
                        if let Some(game) = &mut self.game {
                            let player = ctx.address();
                            game.do_send(Remove {
                                player: player.clone(),
                                x,
                                y,
                            });
                        }
                    }
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
    Remove { x: usize, y: usize },
    Feed { row: [bool; WIDTH] },
}

impl Handler<OutgoingMessage> for Player {
    type Result = ();

    fn handle(&mut self, msg: OutgoingMessage, ctx: &mut Self::Context) {
        if let Ok(msg) = serde_json::to_string(&msg) {
            ctx.text(msg);
        }
    }
}
