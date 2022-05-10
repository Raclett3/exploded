use super::board::Board;
use crate::game::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use yew::prelude::*;

const RANKS: [(usize, &str); 10] = [
    (0, "D"),
    (5000, "D+"),
    (10000, "C"),
    (15000, "C+"),
    (20000, "B"),
    (25000, "B+"),
    (30000, "A"),
    (35000, "A+"),
    (40000, "A++"),
    (50000, "Awesome"),
];

fn fit_with_aspect_ratio(
    width: f64,
    height: f64,
    aspect_width: f64,
    aspect_height: f64,
) -> (f64, f64) {
    if width * aspect_height > height * aspect_width {
        (height * aspect_width / aspect_height, height)
    } else {
        (width, width * aspect_height / aspect_width)
    }
}

fn raf_loop(mut func: impl FnMut() + 'static) {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let request_animation_frame = |window: &web_sys::Window, f: &Closure<dyn FnMut()>| {
        window
            .request_animation_frame(f.as_ref().unchecked_ref())
            .unwrap();
    };

    let window = web_sys::window().unwrap();
    let cloned_window = window.clone();
    *f.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        func();
        request_animation_frame(&cloned_window, g.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));
    request_animation_frame(&window, f.borrow().as_ref().unwrap());
}

struct Audio {
    context: Rc<web_sys::AudioContext>,
    buf: web_sys::AudioBuffer,
}

impl Audio {
    fn new(context: Rc<web_sys::AudioContext>, buf: web_sys::AudioBuffer) -> Self {
        Audio { context, buf }
    }

    fn play(&self) {
        let node = self.context.create_buffer_source().unwrap();
        node.set_buffer(Some(&self.buf));
        node.connect_with_audio_node(&self.context.destination())
            .unwrap();
        node.start().unwrap();
    }
}

struct LazyAudio {
    context: Rc<web_sys::AudioContext>,
    src: String,
    audio: Arc<Mutex<Option<Audio>>>,
}

async fn resolve_promise<T: From<JsValue>>(promise: js_sys::Promise) -> T {
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .unwrap()
        .into()
}

impl LazyAudio {
    fn new(src: &str, context: Rc<web_sys::AudioContext>) -> LazyAudio {
        LazyAudio {
            context,
            src: src.to_string(),
            audio: Arc::new(Mutex::new(None)),
        }
    }

    async fn load(&self) {
        let mut audio = self.audio.lock().unwrap();
        if audio.is_none() {
            let window = web_sys::window().unwrap();
            let res: web_sys::Response = resolve_promise(window.fetch_with_str(&self.src)).await;
            let array_buffer: js_sys::ArrayBuffer =
                resolve_promise(res.array_buffer().unwrap()).await;
            let buffer: web_sys::AudioBuffer =
                resolve_promise(self.context.decode_audio_data(&array_buffer).unwrap()).await;
            *audio = Some(Audio::new(self.context.clone(), buffer));
        }
    }

    async fn play_force(&self) {
        let audio = self.audio.lock().unwrap();
        audio.as_ref().unwrap().play();
    }

    async fn play(&self) {
        let is_loaded = {
            let audio = self.audio.lock().unwrap();
            audio.is_some()
        };

        if is_loaded {
            self.play_force().await;
        } else {
            self.load().await;
            self.play_force().await;
        }
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let game = use_reducer(Game::new);
    let audio_context = use_ref(|| web_sys::AudioContext::new().unwrap());
    let cloned_context = audio_context.clone();
    let break_sound = use_ref(|| LazyAudio::new("/sound/break.wav", cloned_context));
    let cloned_context = audio_context.clone();
    let fall_sound = use_ref(|| LazyAudio::new("/sound/fall.wav", cloned_context));
    let cloned_context = audio_context.clone();
    let feed_sound = use_ref(|| LazyAudio::new("/sound/feed.wav", cloned_context));
    let cloned_context = audio_context.clone();
    let stuck_sound = use_ref(|| LazyAudio::new("/sound/stuck.wav", cloned_context));

    let cloned_game = game.clone();

    let cloned_break = break_sound.clone();
    let cloned_fall = fall_sound.clone();
    let cloned_feed = feed_sound.clone();
    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move { cloned_break.load().await });
            wasm_bindgen_futures::spawn_local(async move { cloned_fall.load().await });
            wasm_bindgen_futures::spawn_local(async move { cloned_feed.load().await });
            game.dispatch(GameAction::Feed);
            raf_loop(move || game.dispatch(GameAction::Animate));
            || ()
        },
        (),
    );

    let window = web_sys::window().unwrap();
    let game = cloned_game;
    let width = window.inner_width().unwrap().as_f64().unwrap();
    let height = window.inner_height().unwrap().as_f64().unwrap();
    let (resized_width, resized_height) =
        fit_with_aspect_ratio(width - 20., height - 20., WIDTH as f64, HEIGHT as f64);
    let cell_size = resized_width as f64 / WIDTH as f64;

    let top = (height - resized_height) / 2.;
    let left = (width - resized_width) / 2.;

    let cloned_game = game.clone();
    let onmousedown = Callback::from(move |event: web_sys::MouseEvent| {
        event.prevent_default();
        let x = ((event.client_x() as f64 - left) / cell_size)
            .max(0.)
            .min(WIDTH as f64 - 1.) as usize;
        let y = ((event.client_y() as f64 - top) / cell_size)
            .max(0.)
            .min(HEIGHT as f64 - 1.) as usize;
        cloned_game.dispatch(GameAction::Remove(x, y));
    });

    let cloned_game = game.clone();
    let ontouchstart = Callback::from(move |event: web_sys::TouchEvent| {
        let touches = event.target_touches();
        for i in 0..touches.length() {
            if let Some(event) = touches.item(i) {
                web_sys::console::log_1(&wasm_bindgen::JsValue::from(event.client_x()));
                web_sys::console::log_1(&wasm_bindgen::JsValue::from(event.client_y()));
                let x = ((event.client_x() as f64 - left) / cell_size)
                    .max(0.)
                    .min(WIDTH as f64 - 1.) as usize;
                let y = ((event.client_y() as f64 - top) / cell_size)
                    .max(0.)
                    .min(HEIGHT as f64 - 1.) as usize;
                cloned_game.dispatch(GameAction::Remove(x, y));
            }
        }
    });

    let (floating_cells, sounds) = if let Some(animator) = &game.animator {
        let animator = animator.borrow();

        if !animator.is_over() {
            let (frame, sound) = animator.frame();
            (Some(frame), sound)
        } else {
            (None, Vec::new())
        }
    } else {
        (None, Vec::new())
    };

    if let Some(sound) = sounds.first() {
        let sound = match sound {
            Sound::Break => break_sound,
            Sound::Feed => feed_sound,
            Sound::Fall => fall_sound,
            Sound::Stuck => stuck_sound,
        };

        wasm_bindgen_futures::spawn_local(async move { sound.play().await });
    }

    let particles = game.particles.borrow().frame();
    let score = game.score_animator.borrow().frame();
    let rank_index = RANKS
        .binary_search_by_key(&(game.score + 1), |x| x.0)
        .unwrap_or_else(|x| x)
        - 1;
    let rank = RANKS[rank_index].1;

    let board = html! {
        <Board<WIDTH, HEIGHT>
            board={game.board.cells}
            floating_cells={floating_cells}
            particles={particles}
            score={score}
            is_game_over={game.is_over()}
            cell_size={cell_size}
            numerator={game.bombs_removed.min(game.bombs_limit)}
            denominator={game.bombs_limit}
            rank={rank} />
    };

    if window.navigator().max_touch_points() > 0 {
        html! {
            <div class="app" ontouchstart={ontouchstart} style={format!("top: {}px; left: {}px;", top, left)}>
                {board}
            </div>
        }
    } else {
        html! {
            <div class="app" onmousedown={onmousedown} style={format!("top: {}px; left: {}px;", top, left)}>
                {board}
            </div>
        }
    }
}
