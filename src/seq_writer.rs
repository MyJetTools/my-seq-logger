use std::time::Duration;

use my_logger::{MyLogEvent, MyLogger, MyLoggerReader};

pub struct SeqWriter {
    pub url: String,
    pub api_key: Option<String>,
    pub max_logs_flush_chunk: usize,
    pub flush_delay: Duration,
    pub app: String,
}

const DEFAULT_FLUSH_SLEEP: u64 = 1;
const DEFAULT_FLUSH_CHUNK: usize = 50;

impl SeqWriter {
    pub fn new(url: String, api_key: Option<String>, app: String) -> Self {
        Self {
            url,
            api_key,
            max_logs_flush_chunk: DEFAULT_FLUSH_CHUNK,
            flush_delay: Duration::from_secs(DEFAULT_FLUSH_SLEEP),
            app,
        }
    }

    pub fn fron_connection_string(connection_string: String, app: String) -> Self {
        if connection_string.to_lowercase().starts_with("http") {
            return Self::new(connection_string, None, app);
        }

        let mut url = None;
        let mut api_key = None;
        let mut max_logs_flush_chunk = DEFAULT_FLUSH_CHUNK;
        let mut flush_delay = DEFAULT_FLUSH_SLEEP;

        for item in connection_string.split(';') {
            let (key, value) = spit_key_value(item);

            match key {
                "Url" => {
                    url = Some(value);
                }
                "ApiKey" => {
                    api_key = Some(value.to_string());
                }
                "FlushLogsChunk" => {
                    max_logs_flush_chunk = value
                        .parse::<usize>()
                        .expect("FlushLogsChunk must be a number");
                }
                "FlushDelay" => {
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
        }
    }

    pub fn start(&self, logger: &mut MyLogger) {
        let logger_reader = logger.get_reader();

        tokio::spawn(read_log(
            logger_reader,
            self.url.clone(),
            self.api_key.clone(),
            self.app.clone(),
            self.max_logs_flush_chunk,
            self.flush_delay,
        ));
    }
}

async fn read_log(
    logger_reader: MyLoggerReader,
    url: String,
    api_key: Option<String>,
    app: String,
    max_logs_flush_chunk: usize,
    flush_delay: Duration,
) {
    loop {
        let events = logger_reader.get_next_line(max_logs_flush_chunk).await;

        match events {
            Some(events) => {
                let stop = flush_events(url.as_str(), api_key.as_ref(), app.as_str(), events).await;

                if stop {
                    break;
                }
            }
            None => {
                tokio::time::sleep(flush_delay).await;
            }
        }
    }
}

async fn flush_events(
    url: &str,
    api_key: Option<&String>,
    app: &str,
    events: Vec<MyLogEvent>,
) -> bool {
    let mut to_flush = Vec::with_capacity(events.len());

    let mut result = false;

    for event in events {
        match event {
            MyLogEvent::NewEvent(event) => {
                to_flush.push(event);
            }
            MyLogEvent::TheEnd => {
                result = true;
            }
        }
    }

    let events_amount = to_flush.len();

    let upload_reusult = super::sdk::push_log_data(url, api_key, app, to_flush).await;

    if let Err(err) = upload_reusult {
        println!(
            "Skipped writing {} log events to seq {}. Err: {:?}",
            events_amount, url, err
        );
    }

    result
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
