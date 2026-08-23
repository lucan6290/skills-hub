use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

impl Default for Setting {
    fn default() -> Self {
        Self {
            key: String::new(),
            value: String::new(),
        }
    }
}
