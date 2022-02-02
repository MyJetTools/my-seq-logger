use my_logger::MyLogEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LogDataContract<'s> {
    #[serde(rename = "@l")]
    pub level: &'s str,
    #[serde(rename = "@t")]
    pub date: String,
    #[serde(rename = "App")]
    pub app: &'s str,
    #[serde(rename = "Process")]
    pub process: String,
    #[serde(rename = "Message")]
    pub message: String,
    #[serde(rename = "Context")]
    pub context: Option<String>,
}

impl<'s> LogDataContract<'s> {
    pub fn from(app: &'s str, src: MyLogEvent) -> Self {
        Self {
            level: src.level.to_string(),
            app,
            context: src.context,
            date: src.dt.to_rfc3339(),
            message: src.message,
            process: src.process,
        }
    }
}
