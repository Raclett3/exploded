use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum RequestMessage {
    Join,
    Leave,
    Remove { x: usize, y: usize },
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ResponseMessage {
    Ready,
    Remove { x: usize, y: usize },
    Feed { row: Vec<bool> },
}
