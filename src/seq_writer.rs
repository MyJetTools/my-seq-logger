use std::{sync::Arc, time::Duration};

use my_logger::{MyLogEvent, MyLoggerReader};
use tokio::sync::Mutex;

const DEFAULT_FLUSH_SLEEP: u64 = 1;
const DEFAULT_FLUSH_CHUNK: usize = 50;

pub struct SeqLogger {
    pub url: String,
    pub api_key: Option<String>,
    pub max_logs_flush_chunk: usize,
    pub flush_delay: Duration,
    pub app: String,
    log_events: Arc<Mutex<Vec<MyLogEvent>>>,
}

impl MyLoggerReader for SeqLogger {
    fn write_log(&self, log_event: MyLogEvent) {
        let log_events = self.log_events.clone();
        tokio::spawn(write_to_log(log_events, log_event));
    }
}

async fn write_to_log(log_events: Arc<Mutex<Vec<MyLogEvent>>>, log_event: MyLogEvent) {
    let mut write_access = log_events.lock().await;
    write_access.push(log_event);
}

impl SeqLogger {
    pub fn new(url: String, api_key: Option<String>, app: String) -> Self {
        Self {
            url,
            api_key,
            max_logs_flush_chunk: DEFAULT_FLUSH_CHUNK,
            flush_delay: Duration::from_secs(DEFAULT_FLUSH_SLEEP),
            app,
            log_events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn from_connection_string(connection_string: &str, app: String) -> Self {
        if connection_string.to_lowercase().starts_with("http") {
            return Self::new(connection_string.to_string(), None, app);
        }

        let mut url = None;
        let mut api_key = None;
        let mut max_logs_flush_chunk = DEFAULT_FLUSH_CHUNK;
        let mut flush_delay = DEFAULT_FLUSH_SLEEP;

        for item in connection_string.split(';') {
            let (key, value) = spit_key_value(item);

            match key {
                "url" => {
                    url = Some(value);
                }
                "apikey" => {
                    api_key = Some(value.to_string());
                }
                "flushlogschunk" => {
                    max_logs_flush_chunk = value
                        .parse::<usize>()
                        .expect("FlushLogsChunk must be a number");
                }
                "flushdelay" => {
                    let value = value.parse::<usize>().expect("FlushDelay must be a number");
                    flush_delay = value as u64;
                }
                _ => {
                    panic!("Invalid key {} of seq connection string ", key);
                }
            }
        }

        if url.is_none() {
            panic!("There is no URL parameter in seq connection string");
        }

        Self {
            url: url.unwrap().to_string(),
            api_key,
            app,
            flush_delay: Duration::from_secs(flush_delay),
            max_logs_flush_chunk,
            log_events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start(&self) {
        tokio::spawn(read_log(
            self.log_events.clone(),
            self.url.clone(),
            self.api_key.clone(),
            self.app.clone(),
            self.max_logs_flush_chunk,
            self.flush_delay,
        ));
    }
}

async fn read_log(
    log_events: Arc<Mutex<Vec<MyLogEvent>>>,
    url: String,
    api_key: Option<String>,
    app: String,
    max_logs_flush_chunk: usize,
    flush_delay: Duration,
) {
    loop {
        let events = {
            let mut events = log_events.lock().await;

            if events.len() == 0 {
                None
            } else if events.len() <= max_logs_flush_chunk {
                let mut result = Vec::new();
                std::mem::swap(&mut result, &mut *events);
                Some(result)
            } else {
                let result = events.drain(..max_logs_flush_chunk).collect();
                Some(result)
            }
        };

        match events {
            Some(events) => {
                flush_events(url.as_str(), api_key.as_ref(), app.as_str(), events).await;
            }
            None => {
                tokio::time::sleep(flush_delay).await;
            }
        }
    }
}

async fn flush_events(url: &str, api_key: Option<&String>, app: &str, events: Vec<MyLogEvent>) {
    let events_amount = events.len();

    let upload_reusult = super::sdk::push_logs_data(url, api_key, app, events).await;

    if let Err(err) = upload_reusult {
        println!(
            "Skipped writing {} log events to seq {}. Err: {:?}",
            events_amount, url, err
        );
    }
}

fn spit_key_value(str: &str) -> (&str, &str) {
    let index = str.find('=');

    if index.is_none() {
        panic!("Invalid {} key value of seq connection string", str);
    }

    let index = index.unwrap();

    return (&str[..index], &str[index + 1..]);
}

#[cfg(test)]
mod tests {
    use super::spit_key_value;

    #[test]
    fn test_split_key_value() {
        let str = "A=B";

        let (key, value) = spit_key_value(str);

        assert_eq!("A", key);
        assert_eq!("B", value);
    }

    #[test]
    fn test_split_key_value_empty_value() {
        let str = "A=";

        let (key, value) = spit_key_value(str);

        assert_eq!("A", key);
        assert_eq!("", value);
    }
}
