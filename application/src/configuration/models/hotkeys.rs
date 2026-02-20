use serde::Deserialize;

#[derive(Deserialize)]
pub struct HotkeysConfig {
    pub filter: String,
    pub pass: String,
}
