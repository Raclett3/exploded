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

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub cell_size: f64,
}

#[function_component(GameHard)]
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

    let board_ref = use_node_ref();
    let game = use_reducer(game::GameHard::new);
    let audio_context = use_ref(|| web_sys::AudioContext::new().unwrap());
    let break_sound = use_sound("/sound/break.wav", &audio_context);
    let fall_sound = use_sound("/sound/fall.wav", &audio_context);
    let feed_sound = use_sound("/sound/feed.wav", &audio_context);
    let stuck_sound = use_sound("/sound/stuck.wav", &audio_context);
    let levelup_sound = use_sound("/sound/levelup.wav", &audio_context);

    let cloned_game = game.clone();

    use_effect_with_deps(
        move |_| {
            raf_loop(move || game.dispatch(GameAction::Animate));
            || ()
        },
        (),
    );

    let position = board_ref.cast::<web_sys::Element>().map(|board| {
        let rect = board.get_bounding_client_rect();
        (rect.x(), rect.y())
    });

    let window = web_sys::window().unwrap();
    let game = cloned_game;

    let cloned_game = game.clone();
    let onmousedown = Callback::from(move |event: web_sys::MouseEvent| {
        event.prevent_default();
        let (left, top) = position.unwrap();
        let x = ((event.client_x() as f64 - left) / cell_size)
            .max(0.)
            .min(WIDTH as f64 - 1.) as usize;
        let y = ((event.client_y() as f64 - top) / cell_size)
            .max(0.)
            .min(HEIGHT as f64 - 1.) as usize;
        cloned_game.dispatch(GameAction::Remove(x, y));
    });

    let cloned_game = game.clone();
    let cloned_board_ref = board_ref.clone();
    let ontouchstart = Callback::from(move |event: web_sys::TouchEvent| {
        let board = cloned_board_ref.cast::<web_sys::Element>().unwrap();
        let rect = board.get_bounding_client_rect();
        let left = rect.x();
        let top = rect.y();
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

    let (floating_cells, mut sounds) = game.board.frame();
    sounds.append(&mut game.sounds());

    for sound in sounds {
        let sound = match sound {
            Sound::Break => &break_sound,
            Sound::Feed => &feed_sound,
            Sound::Fall => &fall_sound,
            Sound::Stuck => &stuck_sound,
            Sound::LevelUp => &levelup_sound,
        };

        let sound = sound.clone();

        wasm_bindgen_futures::spawn_local(async move { sound.play().await });
    }

    let particles = game.board.particles();

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

    let until_single = (game.until_single + 1) as f64 / (game.single_frequency) as f64;
    let indicator_width = (WIDTH as f64 * until_single).to_string();
    let indicator_color = if game.until_single == 0 {
        "#FF2222"
    } else {
        "#FFFFFF"
    };
    let grade = game.grade();
    let (a, b) = game.grade_condition();
    let grade_zoom_rate = game.grade_zoom_rate();
    let timer = game.timer.borrow().frame();
    let debug = cfg!(debug_assertions);

    html! {
        <svg style={format!("transform: scale({cell_size});")} width={width.clone()} height={height.clone()} onmousedown={onmousedown} ontouchstart={ontouchstart} ref={board_ref}>
            <text x={center_x.clone()} y={center_y.clone()} class="numerator" font-size="1px">
                {format!("{:03}", game.level.min(game.level_limit))}
            </text>
            <text x={center_x.clone()} y={center_y} class="denominator" font-size="1px">
                {format!("{:03}", game.level_limit)}
            </text>
            <Board<WIDTH, HEIGHT>
                floating_cells={floating_cells}
                particles={particles} />
            if game.is_over() && !game.board.is_animating() {
                <rect x="0" y="0" width={width.clone()} height={height} fill="rgba(0, 0, 0, 0.5)" />
                <text x={center_x.clone()} y={upper_y.clone()} class="text-center" font-size="1px" dominant-baseline="hanging">{"GAME OVER"}</text>
                <Button x={WIDTH as f64 / 2.} y={HEIGHT as f64 / 3. * 2.} font_size="0.5px" onclick={onclick}>{"Retry"}</Button>
            }
            if !game.is_started {
                <text x={center_x.clone()} y={upper_y} class="text-center" font-size="1px" dominant-baseline="hanging">{"READY"}</text>
            }
            if game.single_frequency < 100 {
                <rect x="0" y="0" width={indicator_width} height="0.1" fill={indicator_color} />
            }
            <text x="0" y="1px" transform={format!("scale({grade_zoom_rate})")} class="grade">
                <tspan font-size="1px">{&grade[0..1]}</tspan>
                <tspan font-size="0.5px">{&grade[1..]}</tspan>
                if debug {
                    <tspan font-size="0.5px">{format!("({a}/{b})")}</tspan>
                }
            </text>
            <text x={width.clone()} y="1" text-anchor="end" class="grade" style="background-color: #900;">
                <tspan font-size="0.5px">{timer}</tspan>
            </text>
        </svg>
    }
}
