# my-seq-logger
Seq Logger


To Setup Seq writer connection string must be specified like

```
Url=http://127.0.0.1:5678;ApiKey=123123;FlushLogsChunk=50;FlushDelay=1
```

Where 

* FlushLogsChunk means the biggest amount of message per upload;
* FlushDelay means max delay between log event happendes and it uploaded
