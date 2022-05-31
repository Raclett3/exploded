use super::{board::AnimatedBoard, WIDTH};
use common::board::CellType;
use std::rc::Rc;
use yew::Reducible;

#[derive(Clone)]
pub struct GameOnline {
    pub board: AnimatedBoard,
}

impl GameOnline {
    pub fn new() -> Self {
        GameOnline {
            board: AnimatedBoard::new(),
        }
    }
}

#[derive(Debug)]
pub enum OnlineGameAction {
    Remove(usize, usize),
    Feed([CellType; WIDTH]),
    Animate,
}

impl Reducible for GameOnline {
    type Action = OnlineGameAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut game = (*self).clone();
        match action {
            OnlineGameAction::Remove(x, y) => {
                game.board.remove(x, y);
                game.board.apply_gravity();
            }
            OnlineGameAction::Feed(row) => {
                game.board.feed(&row);
            }
            OnlineGameAction::Animate => {
                game.board.animate();
            }
        }

        game.into()
    }
}
