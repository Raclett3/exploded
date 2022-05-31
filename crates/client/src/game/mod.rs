mod animation;
mod board;
mod hard;
mod normal;
mod online;

pub use animation::*;
pub use hard::GameHard;
pub use normal::Game;
pub use online::GameOnline;
pub use online::OnlineGameAction;

pub const WIDTH: usize = 8;
pub const HEIGHT: usize = 9;

pub enum GameAction {
    Feed,
    Remove(usize, usize),
    Animate,
    Retry,
}
