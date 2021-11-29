use std::time::Duration;

use my_logger::{MyLogEvent, MyLogger, MyLoggerReader};

pub struct SeqWriter {
    pub url: String,
    pub api_key: Option<String>,
    pub max_logs_flush_chunk: usize,
    pub flush_delay: Duration,
    pub app: String,
}

impl SeqWriter {
    pub fn new(url: String, api_key: Option<String>, app: String) -> Self {
        Self {
            url,
            api_key,
            max_logs_flush_chunk: 10,
            flush_delay: Duration::from_secs(1),
            app,
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
