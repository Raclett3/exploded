use super::board::Board;
use super::button::Button;
use crate::game::{self, *};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{
    atomic::{AtomicBool, Ordering::Relaxed},
    Arc, Mutex,
};
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
    is_loaded: AtomicBool,
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
            is_loaded: AtomicBool::new(false),
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

        self.is_loaded.store(true, Relaxed);
    }

    async fn play(&self) {
        if self.is_loaded.load(Relaxed) {
            let audio = self.audio.lock().unwrap();
            audio.as_ref().unwrap().play();
        }
    }
}

#[derive(Clone, Properties, PartialEq)]
pub struct PreviewProps {
    pub x: f64,
    pub y: f64,
}

#[function_component(Preview)]
pub fn preview(props: &PreviewProps) -> Html {
    let PreviewProps { x, y } = props.clone();
    let x = x.to_string();
    let y = y.to_string();
    html! {
        <rect x={x} y={y} width="1" height="1" opacity="0.2" class="fill" />
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub cell_size: f64,
}

#[function_component(Game)]
pub fn game_component(props: &Props) -> Html {
    let Props { cell_size } = props.clone();
    let use_sound = |src: &str, context: &Rc<web_sys::AudioContext>| {
        let cloned_context = context.clone();
        let sound = use_ref(|| LazyAudio::new(src, cloned_context));
        let cloned_sound = sound.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move { cloned_sound.load().await });
                || ()
            },
            (),
        );
        sound
    };

    let remove_preview = use_state(Vec::new);
    let board_ref = use_node_ref();
    let game = use_reducer(game::Game::new);
    let audio_context = use_ref(|| web_sys::AudioContext::new().unwrap());
    let break_sound = use_sound("/sound/break.wav", &audio_context);
    let fall_sound = use_sound("/sound/fall.wav", &audio_context);
    let feed_sound = use_sound("/sound/feed.wav", &audio_context);
    let stuck_sound = use_sound("/sound/stuck.wav", &audio_context);

    let cloned_game = game.clone();

    use_effect_with_deps(
        move |_| {
            game.dispatch(GameAction::Feed);
            raf_loop(move || game.dispatch(GameAction::Animate));
            || ()
        },
        (),
    );

    let window = web_sys::window().unwrap();
    let game = cloned_game;
    let position = board_ref.cast::<web_sys::Element>().map(|board| {
        let rect = board.get_bounding_client_rect();
        (rect.x(), rect.y())
    });

    let cloned_game = game.clone();
    let cloned_remove_preview = remove_preview.clone();
    let onmousemove = Callback::from(move |event: web_sys::MouseEvent| {
        let (left, top) = position.unwrap();
        if cloned_game.is_over() || cloned_game.board.is_animating() {
            cloned_remove_preview.set(Vec::new());
            return;
        }
        let x = ((event.client_x() as f64 - left) / cell_size)
            .max(0.)
            .min(WIDTH as f64 - 1.) as usize;
        let y = ((event.client_y() as f64 - top) / cell_size)
            .max(0.)
            .min(HEIGHT as f64 - 1.) as usize;
        let mut board = cloned_game.board.board.clone();
        let removed_cells = board.remove(x, y);
        let preview = removed_cells
            .into_iter()
            .map(|(_, _, x, y, _)| (x as f64, y as f64))
            .collect();
        cloned_remove_preview.set(preview);
    });

    let cloned_game = game.clone();
    let cloned_remove_preview = remove_preview.clone();
    let onmousedown = Callback::from(move |event: web_sys::MouseEvent| {
        let (left, top) = position.unwrap();
        cloned_remove_preview.set(Vec::new());
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
    let cloned_remove_preview = remove_preview.clone();
    let ontouchstart = Callback::from(move |event: web_sys::TouchEvent| {
        cloned_remove_preview.set(Vec::new());
        let (left, top) = position.unwrap();
        let touches = event.target_touches();
        for i in 0..touches.length() {
            if let Some(event) = touches.item(i) {
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

    let cloned_game = game.clone();
    let onclick = Callback::from(move |_| cloned_game.dispatch(GameAction::Retry));

    let (floating_cells, sounds) = game.board.frame();

    if let Some(sound) = sounds.first() {
        let sound = match sound {
            Sound::Break => break_sound,
            Sound::Feed => feed_sound,
            Sound::Fall => fall_sound,
            Sound::Stuck => stuck_sound,
            _ => unreachable!(),
        };

        wasm_bindgen_futures::spawn_local(async move { sound.play().await });
    }

    let particles = game.board.particles();
    let score = game.score_animator.borrow().frame();
    let rank_index = RANKS
        .binary_search_by_key(&(game.score + 1), |x| x.0)
        .unwrap_or_else(|x| x)
        - 1;
    let rank = RANKS[rank_index].1;

    let (onmousedown, ontouchstart) = if window.navigator().max_touch_points() > 0 {
        (Callback::from(|_| ()), ontouchstart)
    } else {
        (onmousedown, Callback::from(|_| ()))
    };

    let width = WIDTH.to_string();
    let height = HEIGHT.to_string();
    let center_x = (WIDTH as f64 / 2.).to_string();
    let center_y = (HEIGHT as f64 / 2.).to_string();
    let upper_y = (HEIGHT as f64 / 3.).to_string();

    let preview = remove_preview.iter().map(|&(x, y)| {
        html! {
            <Preview x={x} y={y} />
        }
    });
    let score_preview = (remove_preview.len() + 1) * remove_preview.len() / 2;

    html! {
        <svg style={format!("transform: scale({cell_size});")} width={width.clone()} height={height.clone()} onmousedown={onmousedown} ontouchstart={ontouchstart} onmousemove={onmousemove} ref={board_ref}>
            <text x={center_x.clone()} y={center_y.clone()} class="numerator" font-size="1px">
                {format!("{:03}", game.bombs_removed.min(game.bombs_limit))}
            </text>
            <text x={center_x.clone()} y={center_y.clone()} class="denominator" font-size="1px">
                {format!("{:03}", game.bombs_limit)}
            </text>
            {for preview}
            <Board<WIDTH, HEIGHT>
                floating_cells={floating_cells}
                particles={particles} />
            if game.is_over() && !game.board.is_animating() {
                <rect x="0" y="0" width={width} height={height} fill="rgba(0, 0, 0, 0.5)" />
                <text x={center_x.clone()} y={upper_y} class="text-center" font-size="1px" dominant-baseline="hanging">{"GAME OVER"}</text>
                <text x={center_x} y={center_y} class="text-center" dominant-baseline="baseline">
                    <tspan font-size="0.5px">{"RANK:"}</tspan>
                    <tspan font-size="1px">{rank}</tspan>
                </text>
                <Button x={WIDTH as f64 / 2.} y={HEIGHT as f64 / 3. * 2.} font_size="0.5px" onclick={onclick}>{"Retry"}</Button>
            }
            <text x="0" y="0" class="text" font-size="0.5px">
                <tspan>{format!("SCORE: {}", score)}</tspan>
                if score_preview > 0 {
                    <tspan dx="0.1em" opacity="0.6">{format!("+{score_preview}")}</tspan>
                }
            </text>
        </svg>
    }
}
