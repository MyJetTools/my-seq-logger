# my-seq-logger
Seq Logger


### Connection string

To Setup Seq writer connection string must be specified like

```
Url=http://127.0.0.1:5678;ApiKey=123123;FlushLogsChunk=50;FlushDelay=1
```

Where 

* FlushLogsChunk means the biggest amount of message per upload;
* FlushDelay means max delay between log event happendes and it uploaded


### How to kick off the Seq writer

#### Include my-logger to Cargo.toml
```toml
[dependencies]
my-logger = { branch = "main", git = "https://github.com/MyJetTools/my-logger.git" }
```
#### Create MyLogger and SeqWriter instances and configure them together
```rust

#[tokio::main]
async fn main() {


let logger = MyLogger::new();

let seq_writer = SeqWriter::from_connection_string("conn_stirng", "app-name");

//Start publishing to seq
seq_writer.start(&logger);


///// Doing some program
///// and writing logs...

logger.write_log(....);


//Stop publishing to seq
logger.shutdown();


}



```
