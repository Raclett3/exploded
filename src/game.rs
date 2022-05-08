use crate::animation::*;
use crate::board::{Board as GameBoard, Cell, CellType};
use rand::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use yew::Reducible;

pub const WIDTH: usize = 8;
pub const HEIGHT: usize = 9;

const PARTICLE_COLORS: [&str; 7] = [
    "#FF0000", "#FF8800", "#FFFF00", "#00FF00", "#00FFFF", "#0000FF", "#FF00FF",
];

#[derive(Clone)]
struct BombGenerator {
    shuffled: Vec<(usize, usize)>,
    rng: StdRng,
}

impl BombGenerator {
    fn new() -> Self {
        let random = js_sys::Math::random();
        let rng = StdRng::seed_from_u64(u64::from_be_bytes(random.to_be_bytes()));
        let mut generator = BombGenerator {
            shuffled: Vec::new(),
            rng,
        };
        generator.shuffle();
        generator
    }

    fn shuffle(&mut self) {
        let mut shuffled = (0..WIDTH)
            .flat_map(|first| (first + 1..WIDTH).map(move |second| (first, second)))
            .collect::<Vec<_>>();
        shuffled.shuffle(&mut self.rng);
        self.shuffled = shuffled;
    }

    fn next(&mut self) -> (usize, usize) {
        self.shuffled.pop().unwrap_or_else(|| {
            self.shuffle();
            self.next()
        })
    }
}

#[derive(Clone, PartialEq)]
pub struct FloatingCell {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub cell_type: CellType,
    pub opacity: f64,
}

fn interpolation((from, to): (f64, f64), position: f64) -> f64 {
    (from * (1. - position)) + (to * position)
}

struct CellAnimator {
    id: usize,
    x: f64,
    y: (f64, f64),
    opacity: (f64, f64),
    delay: usize,
    duration: usize,
    elapsed: usize,
    cell_type: CellType,
}

impl CellAnimator {
    fn new(
        id: usize,
        x: f64,
        y: (f64, f64),
        opacity: (f64, f64),
        delay: usize,
        duration: usize,
        cell_type: CellType,
    ) -> Self {
        CellAnimator {
            id,
            x,
            y,
            opacity,
            delay,
            duration,
            cell_type,
            elapsed: 0,
        }
    }
}

impl Animation<FloatingCell> for CellAnimator {
    fn advance_frames(&mut self, frames: usize) {
        self.elapsed += frames;
    }

    fn current_frame(&self) -> FloatingCell {
        let relative_time = self.elapsed.saturating_sub(self.delay).min(self.duration) as f64
            / self.duration as f64;
        FloatingCell {
            id: self.id,
            x: self.x,
            y: interpolation(self.y, relative_time),
            cell_type: self.cell_type,
            opacity: interpolation(self.opacity, relative_time),
        }
    }

    fn is_over(&self) -> bool {
        self.duration + self.delay <= self.elapsed
    }
}

#[derive(Clone, PartialEq)]
pub struct FloatingParticle {
    pub id: usize,
    pub color: &'static str,
    pub cell_type: CellType,
    pub x: f64,
    pub y: f64,
    pub expansion: f64,
    pub opacity: f64,
}

struct ParticleAnimator {
    id: usize,
    color: &'static str,
    cell_type: CellType,
    x: f64,
    y: f64,
    expansion: (f64, f64),
    opacity: (f64, f64),
    delay: usize,
    duration: usize,
    elapsed: usize,
}

impl ParticleAnimator {
    #[allow(clippy::too_many_arguments)]
    fn new(
        id: usize,
        color: &'static str,
        cell_type: CellType,
        x: f64,
        y: f64,
        expansion: (f64, f64),
        opacity: (f64, f64),
        delay: usize,
        duration: usize,
    ) -> Self {
        ParticleAnimator {
            id,
            color,
            cell_type,
            x,
            y,
            expansion,
            opacity,
            delay,
            duration,
            elapsed: 0,
        }
    }
}

impl Animation<FloatingParticle> for ParticleAnimator {
    fn advance_frames(&mut self, frames: usize) {
        self.elapsed += frames;
    }

    fn current_frame(&self) -> FloatingParticle {
        let relative_time = self.elapsed.saturating_sub(self.delay).min(self.duration) as f64
            / self.duration as f64;
        FloatingParticle {
            id: self.id,
            color: self.color,
            cell_type: self.cell_type,
            x: self.x,
            y: self.y,
            expansion: interpolation(self.expansion, relative_time),
            opacity: interpolation(self.opacity, relative_time),
        }
    }

    fn is_over(&self) -> bool {
        self.duration + self.delay <= self.elapsed
    }
}

pub struct NumberAnimator {
    target: usize,
    current: usize,
}

impl NumberAnimator {
    fn new(target: usize) -> Self {
        NumberAnimator { target, current: 0 }
    }

    fn set_target(&mut self, target: usize) {
        self.target = target;
    }
}

impl Animation<usize> for NumberAnimator {
    fn advance_frames(&mut self, frames: usize) {
        for _ in 0..frames {
            self.current = (self.current * 3 + self.target + 3) / 4;
        }
    }

    fn current_frame(&self) -> usize {
        self.current
    }

    fn is_over(&self) -> bool {
        false
    }
}

#[derive(Clone)]
pub struct Game {
    pub board: GameBoard<WIDTH, HEIGHT>,
    generator: BombGenerator,
    pub score: usize,
    pub bombs_removed: usize,
    pub bombs_limit: usize,
    #[allow(clippy::type_complexity)]
    pub animator:
        Option<Rc<RefCell<FloatAnimator<Vec<FloatingCell>, AnimationChain<Vec<FloatingCell>>>>>>,
    pub particles:
        Rc<RefCell<FloatAnimator<Vec<FloatingParticle>, EndlessAnimator<FloatingParticle>>>>,
    pub score_animator: Rc<RefCell<FloatAnimator<usize, NumberAnimator>>>,
}

pub enum GameAction {
    Feed,
    Remove(usize, usize),
    Animate,
}

impl Game {
    pub fn new() -> Self {
        Game {
            board: GameBoard::new(),
            generator: BombGenerator::new(),
            score: 0,
            bombs_removed: 0,
            bombs_limit: 999,
            animator: None,
            particles: Rc::new(RefCell::new(FloatAnimator::new(EndlessAnimator::new(
                Vec::new(),
            )))),
            score_animator: Rc::new(RefCell::new(FloatAnimator::new(NumberAnimator::new(0)))),
        }
    }

    fn feed(&mut self) {
        let bombs = self.generator.next();
        let mut row = [CellType::Tile; WIDTH];
        row[bombs.0] = CellType::Bomb;
        row[bombs.1] = CellType::Bomb;
        self.board.feed(&row);
    }

    pub fn is_over(&self) -> bool {
        let is_filled = self
            .board
            .cells
            .iter()
            .any(|x| x.first().cloned().flatten().is_some());
        let reached_limit = self.bombs_limit <= self.bombs_removed;
        is_filled || reached_limit
    }
}

impl Reducible for Game {
    type Action = GameAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut self_cloned = (*self).clone();

        match action {
            GameAction::Remove(x, y) => {
                if self_cloned.is_over() {
                    let mut game = Game::new();
                    game.feed();
                    return Rc::new(game);
                }

                let dists = self_cloned.board.remove(x, y);
                if !dists.is_empty() {
                    self_cloned.score += (dists.len() + 1) * dists.len() / 2;
                    self_cloned
                        .score_animator
                        .borrow_mut()
                        .animator
                        .set_target(self_cloned.score);
                    let bombs = dists.iter().filter(|x| x.4 == CellType::Bomb).count();
                    self_cloned.bombs_removed += bombs;

                    {
                        let mut particle_animator = self_cloned.particles.borrow_mut();
                        dists
                            .iter()
                            .map(|&(id, dist, x, y, cell_type)| {
                                let (color, expansion, duration) = match cell_type {
                                    CellType::Bomb => (PARTICLE_COLORS[dist % 7], (0., 3.), 40),
                                    CellType::Tile => ("#FFFFFF", (0., 1.), 10),
                                };
                                ParticleAnimator::new(
                                    id + 1_000_000,
                                    color,
                                    cell_type,
                                    x as f64,
                                    y as f64,
                                    expansion,
                                    (1., 0.),
                                    dist * 3,
                                    duration,
                                )
                            })
                            .for_each(|x| particle_animator.animator.push(x));
                    }

                    let remove_animation = self_cloned
                        .board
                        .cells
                        .iter()
                        .enumerate()
                        .flat_map(|(x, col)| {
                            col.iter().enumerate().flat_map(move |(y, cell)| {
                                cell.map(|cell| {
                                    let Cell { cell_type, id } = cell;
                                    Box::new(CellAnimator::new(
                                        id,
                                        x as f64,
                                        (y as f64, y as f64),
                                        (1., 1.),
                                        0,
                                        1,
                                        cell_type,
                                    ))
                                        as Box<dyn Animation<FloatingCell>>
                                })
                            })
                        })
                        .chain(dists.iter().map(|&(id, dist, x, y, cell_type)| {
                            Box::new(CellAnimator::new(
                                id,
                                x as f64,
                                (y as f64, y as f64),
                                (1., 0.),
                                dist * 3,
                                10,
                                cell_type,
                            )) as Box<dyn Animation<FloatingCell>>
                        }))
                        .collect();
                    let dists = self_cloned.board.apply_gravity();
                    let cells_animation = self_cloned
                        .board
                        .cells
                        .iter()
                        .enumerate()
                        .flat_map(|(x, col)| {
                            let dists = dists.clone();
                            col.iter().enumerate().flat_map(move |(y, cell)| {
                                cell.map(|cell| {
                                    let Cell { id, cell_type } = cell;
                                    let dist = dists.get(&id).cloned().unwrap_or(0);
                                    Box::new(CellAnimator::new(
                                        id,
                                        x as f64,
                                        ((y - dist) as f64, y as f64),
                                        (1., 1.),
                                        0,
                                        dist * 5 + 1,
                                        cell_type,
                                    ))
                                        as Box<dyn Animation<FloatingCell>>
                                })
                            })
                        })
                        .collect();
                    self_cloned.feed();
                    let feed_animation = self_cloned
                        .board
                        .cells
                        .iter()
                        .enumerate()
                        .flat_map(|(x, col)| {
                            col.iter().enumerate().flat_map(move |(y, cell)| {
                                cell.map(|cell| {
                                    let Cell { id, cell_type } = cell;
                                    Box::new(CellAnimator::new(
                                        id,
                                        x as f64,
                                        ((y + 1) as f64, y as f64),
                                        (1., 1.),
                                        0,
                                        10,
                                        cell_type,
                                    ))
                                        as Box<dyn Animation<FloatingCell>>
                                })
                            })
                        })
                        .collect();

                    let animation = AnimationChain::new(vec![
                        Box::new(Animator::new(remove_animation)),
                        Box::new(Animator::new(cells_animation)),
                        Box::new(Animator::new(feed_animation)),
                    ]);
                    self_cloned.animator =
                        Some(Rc::new(RefCell::new(FloatAnimator::new(animation))));
                }
            }
            GameAction::Feed => {
                self_cloned.feed();
            }
            GameAction::Animate => {
                if let Some(animator) = &self.animator {
                    animator.borrow_mut().animate();
                }
                self.particles.borrow_mut().animate();
                self.score_animator.borrow_mut().animate();
            }
        }

        self_cloned.into()
    }
}
