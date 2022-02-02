use crate::contracts::LogDataContract;
use flurl::FlUrl;
use hyper::Error;
use my_logger::MyLogEvent;

pub async fn push_logs_data(
    url: &str,
    api_key: Option<&String>,
    app: &str,
    data: Vec<MyLogEvent>,
) -> Result<(), Error> {
    let body = complie_body(app, data);

    let mut fl_url = FlUrl::new(url);

    if let Some(api_key) = api_key {
        fl_url = fl_url.with_header("X-Seq-ApiKey", api_key);
    };

    fl_url.set_query_param("clef").post(Some(body)).await?;

    Ok(())
}

fn complie_body(app: &str, data: Vec<MyLogEvent>) -> Vec<u8> {
    let mut result = Vec::new();

    for log_data in data {
        let contract = LogDataContract::from(app, log_data);

        let item = serde_json::to_vec(&contract).unwrap();
        result.extend(item);
    }

    result
}
