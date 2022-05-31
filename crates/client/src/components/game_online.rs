use super::board::Board;
use crate::game::{self, *};
use crate::websocket::WebsocketBus;
use common::board::CellType;
use common::model::{RequestMessage, ResponseMessage};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{
    atomic::{AtomicBool, Ordering::Relaxed},
    Arc, Mutex,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew_agent::use_bridge;

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

#[function_component(GameOnline)]
pub fn game_online(props: &Props) -> Html {
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
    let game = use_reducer(game::GameOnline::new);
    let audio_context = use_ref(|| web_sys::AudioContext::new().unwrap());
    let break_sound = use_sound("/sound/break.wav", &audio_context);
    let fall_sound = use_sound("/sound/fall.wav", &audio_context);
    let feed_sound = use_sound("/sound/feed.wav", &audio_context);
    let stuck_sound = use_sound("/sound/stuck.wav", &audio_context);
    let levelup_sound = use_sound("/sound/levelup.wav", &audio_context);

    let cloned_game = game.clone();
    let websocket = use_bridge::<WebsocketBus, _>(move |msg: ResponseMessage| {
        let action = match msg {
            ResponseMessage::Remove { x, y } => OnlineGameAction::Remove(x, y),
            ResponseMessage::Feed { row } => {
                if let Ok(row) = <[bool; WIDTH]>::try_from(row) {
                    let row = row.map(|x| if x { CellType::Bomb } else { CellType::Tile });
                    OnlineGameAction::Feed(row)
                } else {
                    return;
                }
            }
            _ => return,
        };
        cloned_game.dispatch(action);
    });

    let cloned_game = game.clone();

    let cloned_ws = websocket.clone();
    use_effect_with_deps(
        move |_| {
            raf_loop(move || game.dispatch(OnlineGameAction::Animate));
            cloned_ws.send(RequestMessage::Join);
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

    let cloned_ws = websocket.clone();
    let onmousedown = Callback::from(move |event: web_sys::MouseEvent| {
        event.prevent_default();
        let (left, top) = position.unwrap();
        let x = ((event.client_x() as f64 - left) / cell_size)
            .max(0.)
            .min(WIDTH as f64 - 1.) as usize;
        let y = ((event.client_y() as f64 - top) / cell_size)
            .max(0.)
            .min(HEIGHT as f64 - 1.) as usize;
        cloned_ws.send(RequestMessage::Remove{x, y});
    });

    let cloned_ws = websocket.clone();
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
                cloned_ws.send(RequestMessage::Remove{x, y});
            }
        }
    });

    let (floating_cells, sounds) = game.board.frame();

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

    html! {
        <svg style={format!("transform: scale({cell_size});")} width={width.clone()} height={height.clone()} onmousedown={onmousedown} ontouchstart={ontouchstart} ref={board_ref}>
            <Board<WIDTH, HEIGHT>
                floating_cells={floating_cells}
                particles={particles} />
        </svg>
    }
}
