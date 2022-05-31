use common::model::{RequestMessage, ResponseMessage};
use futures::{
    channel::mpsc::{channel, Sender},
    SinkExt, StreamExt,
};
use gloo_net::websocket::{futures::WebSocket, Message};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use wasm_bindgen_futures::spawn_local;
use yew_agent::{Agent, AgentLink, Context, HandlerId};

struct WebsocketHandler {
    req_tx: Sender<RequestMessage>,
}

impl WebsocketHandler {
    fn new(mut callback: impl FnMut(ResponseMessage) + 'static) -> Self {
        let (tx, mut rx) = channel(0);

        let ws = WebSocket::open("ws://localhost:9000/ws").unwrap();
        let (mut write, mut read) = ws.split();
        spawn_local(async move {
            while let Some(msg) = rx.next().await {
                write
                    .send(Message::Text(serde_json::to_string(&msg).unwrap()))
                    .await
                    .unwrap();
            }
        });
        spawn_local(async move {
            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(text)) = msg {
                    if let Ok(msg) = serde_json::from_str(&text) {
                        callback(msg);
                    }
                }
            }
        });

        Self { req_tx: tx }
    }

    fn send(&mut self, msg: RequestMessage) {
        let _ = self.req_tx.try_send(msg);
    }
}

pub struct WebsocketBus {
    subscribers: Arc<Mutex<HashSet<HandlerId>>>,
    handler: WebsocketHandler,
}

impl Agent for WebsocketBus {
    type Reach = Context<Self>;
    type Message = ();
    type Input = RequestMessage;
    type Output = ResponseMessage;

    fn create(link: AgentLink<Self>) -> Self {
        let subscribers = Arc::new(Mutex::new(HashSet::new()));

        let subscribers_cloned = subscribers.clone();
        let handler = WebsocketHandler::new(move |msg: ResponseMessage| {
            for sub in subscribers_cloned.lock().unwrap().iter() {
                link.respond(*sub, msg.clone());
            }
        });
        Self {
            subscribers,
            handler,
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        self.handler.send(msg);
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.lock().unwrap().insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.lock().unwrap().remove(&id);
    }
}
