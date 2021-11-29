use serde::{Deserialize, Serialize};

use crate::sdk::LogData;

#[derive(Serialize, Deserialize, Debug)]
pub struct LogDataContract<'s> {
    #[serde(rename = "@l")]
    pub level: &'s str,
    #[serde(rename = "@t")]
    pub date: String,
    #[serde(rename = "App")]
    pub app: String,
    #[serde(rename = "Process")]
    pub process: String,
    #[serde(rename = "Message")]
    pub message: String,
    #[serde(rename = "Context")]
    pub conext: Option<String>,
}

impl<'s> Into<LogDataContract<'s>> for LogData {
    fn into(self) -> LogDataContract<'s> {
        LogDataContract {
            level: self.level.to_string(),
            app: self.app,
            conext: self.conext,
            date: self.date.to_rfc3339(),
            message: self.message,
            process: self.process,
        }
    }
}
