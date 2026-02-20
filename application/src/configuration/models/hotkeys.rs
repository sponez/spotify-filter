use serde::Deserialize;

#[derive(Deserialize)]
pub struct HotkeysConfig {
    pub discard: String,
    pub like: String,
}
