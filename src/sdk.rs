use crate::contracts::LogDataContract;
use flurl::FlUrl;
use hyper::Error;
use rust_extensions::date_time::DateTimeAsMicroseconds;

pub enum LogLevel {
    FatalError,
    Error,
    Warning,
    Info,
}

impl LogLevel {
    pub fn to_string(&self) -> &'static str {
        match self {
            LogLevel::FatalError => "FatalError",
            LogLevel::Error => "Error",
            LogLevel::Warning => "Warning",
            LogLevel::Info => "Info",
        }
    }
}

pub struct LogData {
    pub level: LogLevel,
    pub date: DateTimeAsMicroseconds,
    pub app: String,
    pub process: String,
    pub message: String,
    pub conext: Option<String>,
}

pub async fn push_log_data(url: &str, api_key: &str, data: Vec<LogData>) -> Result<(), Error> {
    let body = complie_body(data);

    FlUrl::new(url)
        .with_header("X-Seq-ApiKey", api_key)
        .set_query_param("clef")
        .post(Some(body))
        .await?;

    Ok(())
}

fn complie_body(data: Vec<LogData>) -> Vec<u8> {
    let mut result = Vec::new();

    for log_data in data {
        let contract: LogDataContract = log_data.into();

        let item = serde_json::to_vec(&contract).unwrap();
        result.extend(item);
    }

    result
}
