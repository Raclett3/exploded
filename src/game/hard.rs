use super::{
    animation::*,
    board::{AnimatedBoard, VisibleState::*},
    GameAction, WIDTH,
};
use crate::animation::{Animation, FloatAnimator};
use crate::board::CellType;
use rand::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use yew::Reducible;

fn cumulate(iter: impl Iterator<Item = usize>) -> impl Iterator<Item = usize> {
    iter.scan(0, |acc, x| {
        *acc += x;
        Some(*acc)
    })
}

#[derive(Clone)]
struct SpreadBombGenerator {
    rng: StdRng,
    generated: [usize; WIDTH],
}

impl SpreadBombGenerator {
    fn new() -> Self {
        let random = js_sys::Math::random();
        let rng = StdRng::seed_from_u64(u64::from_be_bytes(random.to_be_bytes()));
        SpreadBombGenerator {
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

    fn reset(&mut self) {
        self.generated = [0; WIDTH];
    }
}

pub struct Grade {
    grade: &'static str,
    decay_rate: usize,
    required_score: isize,
}

impl Grade {
    const fn new(grade: &'static str, decay_rate: usize, required_score: isize) -> Grade {
        Grade {
            grade,
            decay_rate,
            required_score,
        }
    }
}

static RANKS: [Grade; 19] = [
    Grade::new("C3", 120, 75),
    Grade::new("C2", 90, 75),
    Grade::new("C1", 90, 100),
    Grade::new("B3", 75, 85),
    Grade::new("B2", 75, 85),
    Grade::new("B1", 75, 130),
    Grade::new("A3", 60, 100),
    Grade::new("A2", 60, 100),
    Grade::new("A1", 60, 200),
    Grade::new("S1", 50, 150),
    Grade::new("S2", 45, 150),
    Grade::new("S3", 40, 150),
    Grade::new("S4", 35, 150),
    Grade::new("S5", 30, 150),
    Grade::new("S6", 25, 150),
    Grade::new("S7", 20, 200),
    Grade::new("S8", 15, 200),
    Grade::new("S9", 10, 250),
    Grade::new("master", 10, 1000000),
];

pub struct GradeManager {
    score: isize,
    current_grade: usize,
    max_chain_per_section: [usize; 10],
    elapsed_frames: f64,
    last_timestamp: f64,
}

impl GradeManager {
    fn new() -> GradeManager {
        GradeManager {
            score: 0,
            current_grade: 0,
            max_chain_per_section: [0; 10],
            elapsed_frames: 0.,
            last_timestamp: js_sys::Date::now(),
        }
    }

    fn decay(&mut self) {
        let now = js_sys::Date::now();
        self.elapsed_frames += (now - self.last_timestamp) / 1000. * 60.;
        self.last_timestamp = now;

        let decay_rate = self.current_grade().decay_rate;
        let decay = self.elapsed_frames as usize / decay_rate;
        self.score -= decay as isize;
        self.elapsed_frames -= (decay * decay_rate) as f64;

        if self.current_grade == 0 && self.score < 0 {
            self.score = 0;
        }
    }

    fn add(&mut self, section: usize, bombs: usize) -> bool {
        fn sqrt(n: usize) -> usize {
            (0..).take_while(|x| x * x <= n).last().unwrap()
        }

        if self.current_grade().grade == "S9" && section == 9 && self.max_chain_per_section[9] > 0 {
            return false;
        }

        let score = sqrt(bombs * bombs * bombs) * (section / 2 + 1);

        self.score += score as isize;
        self.max_chain_per_section[section] = self.max_chain_per_section[section].max(bombs);

        let is_promoted = self.score >= self.current_grade().required_score;
        while self.score >= self.current_grade().required_score {
            if (section < 9 || !self.fulfills_master_condition())
                && self.current_grade().grade == "S9"
            {
                return false;
            }
            self.score -= self.current_grade().required_score;
            self.current_grade += 1;
        }

        is_promoted
    }

    fn current_grade(&self) -> &'static Grade {
        &RANKS[self.current_grade]
    }

    fn fulfills_master_condition(&self) -> bool {
        let section_conditions = self.max_chain_per_section[0..=8].iter().all(|&x| x >= 5);
        let current_grade = self.current_grade();
        current_grade.grade == "S9"
            && self.score >= current_grade.required_score
            && section_conditions
    }
}

struct GradeAnimation {
    frame_since_promotion: usize,
}

impl GradeAnimation {
    fn new() -> Self {
        GradeAnimation {
            frame_since_promotion: 30,
        }
    }

    fn promote(&mut self) {
        self.frame_since_promotion = 0;
    }
}

impl Animation<f64> for GradeAnimation {
    fn advance_frames(&mut self, frames: usize) {
        self.frame_since_promotion += frames;
    }

    fn current_frame(&self) -> f64 {
        2. - self.frame_since_promotion.min(30) as f64 / 30.0 * 1.
    }

    fn is_over(&self) -> bool {
        false
    }
}

pub struct Timer {
    pre_timer_frames: usize,
    elapsed_frames: usize,
}

impl Timer {
    fn new(pre_timer_frames: usize) -> Self {
        Timer {
            pre_timer_frames,
            elapsed_frames: 0,
        }
    }

    fn is_started(&self) -> bool {
        self.elapsed_frames >= self.pre_timer_frames
    }
}

impl Animation<String> for Timer {
    fn advance_frames(&mut self, frames: usize) {
        self.elapsed_frames += frames;
    }

    fn current_frame(&self) -> String {
        let frames = self.elapsed_frames.saturating_sub(self.pre_timer_frames);
        let centiseconds = frames % 60 * 100 / 60;
        let seconds = frames / 60 % 60;
        let minutes = frames / 3600 % 60;
        format!("{minutes:02}:{seconds:02}:{centiseconds:02}")
    }

    fn is_over(&self) -> bool {
        false
    }
}

#[derive(Clone)]
pub struct GameHard {
    pub board: AnimatedBoard,
    generator: SpreadBombGenerator,
    grade: Rc<RefCell<GradeManager>>,
    pub until_single: usize,
    pub single_frequency: usize,
    pub section: usize,
    pub level: usize,
    pub level_limit: usize,
    pub timer: Rc<RefCell<FloatAnimator<String, Timer>>>,
    pub is_started: bool,
    sounds: Rc<RefCell<Vec<Sound>>>,
    grade_animation: Rc<RefCell<FloatAnimator<f64, GradeAnimation>>>,
}

impl GameHard {
    pub fn new() -> Self {
        GameHard {
            board: AnimatedBoard::new(),
            generator: SpreadBombGenerator::new(),
            grade: Rc::new(RefCell::new(GradeManager::new())),
            until_single: 999,
            single_frequency: 999,
            section: 0,
            level: 0,
            level_limit: 999,
            timer: Rc::new(RefCell::new(FloatAnimator::new(Box::new(Timer::new(60))))),
            is_started: false,
            sounds: Rc::new(RefCell::new(Vec::new())),
            grade_animation: Rc::new(RefCell::new(FloatAnimator::new(Box::new(
                GradeAnimation::new(),
            )))),
        }
    }

    pub fn is_over(&self) -> bool {
        let reached_limit = self.level >= self.level_limit;
        self.board.is_filled() || reached_limit
    }

    pub fn next_row(&mut self) -> [CellType; WIDTH] {
        if self.level % 100 != 99 && self.level != 998 && self.grade() != "master" {
            self.level += 1;
        }

        if self.until_single == 0 {
            self.until_single = self.single_frequency - 1;
            let bomb = self.generator.next_single();
            let mut row = [CellType::Tile; WIDTH];
            row[bomb] = CellType::Bomb;
            row
        } else {
            let bombs = self.generator.next_double();
            let mut row = [CellType::Tile; WIDTH];
            row[bombs.0] = CellType::Bomb;
            row[bombs.1] = CellType::Bomb;
            self.until_single -= 1;
            row
        }
    }

    pub fn grade(&self) -> &'static str {
        let grade = self.grade.borrow().current_grade().grade;
        if grade == "master" && self.level >= self.level_limit && !self.board.is_filled() {
            "Grandmaster"
        } else {
            grade
        }
    }

    pub fn grade_condition(&self) -> (isize, isize) {
        let grade = self.grade.borrow();
        (grade.score, grade.current_grade().required_score)
    }

    pub fn sounds(&self) -> Vec<Sound> {
        std::mem::take(self.sounds.borrow_mut().as_mut())
    }

    pub fn grade_zoom_rate(&self) -> f64 {
        self.grade_animation.borrow().frame()
    }
}

pub const SINGLE_FREQUENCY: [usize; 10] = [9999, 9, 8, 7, 6, 5, 4, 3, 2, 2];

impl Reducible for GameHard {
    type Action = GameAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut game = (*self).clone();

        match action {
            GameAction::Remove(x, y) => {
                if game.is_over() {
                    return game.into();
                }

                let (removed_cells, removed_bombs) = game.board.remove(x, y);
                if removed_cells > 0 {
                    game.level += removed_bombs;
                    let section = (game.level / 100).min(9);

                    let is_promoted = game.grade.borrow_mut().add(section, removed_bombs);

                    if game.section < section {
                        game.section = section;
                        game.single_frequency = SINGLE_FREQUENCY[section];
                        game.until_single = game.single_frequency;
                        if game.section == 9 && game.grade() == "master" {
                            game.level = 900;
                            game.board.reset();
                            game.generator.reset();
                            game.board.visible = Invisible;
                        }
                    }

                    game.board.apply_gravity();
                    let row = game.next_row();
                    game.board.feed(&row);

                    if is_promoted || game.grade() == "Grandmaster" {
                        self.sounds.borrow_mut().push(Sound::LevelUp);
                        self.grade_animation.borrow_mut().animation.promote();
                    }

                    if game.is_over() && game.board.visible == Invisible {
                        game.board.visible = InvisibleWhileAnimation;
                    }
                }
            }
            GameAction::Feed => {
                let row = game.next_row();
                game.board.feed(&row);
            }
            GameAction::Animate => {
                game.board.animate();
                game.grade_animation.borrow_mut().animate();
                game.grade.borrow_mut().decay();
                if !game.is_over() || game.board.is_animating() {
                    game.timer.borrow_mut().animate();
                }
                if !game.is_started && game.timer.borrow().animation.is_started() {
                    game.is_started = true;
                    let row = game.next_row();
                    game.board.feed(&row);
                }
            }

            GameAction::Retry => {
                return Rc::new(GameHard::new());
            }
        }

        game.into()
    }
}
