<img src="https://github.com/user-attachments/assets/46a5c546-7e9b-42c7-87f4-bc8defe674e0" width=250 />

# DuckDB Pyroscope Extension
This experimental extension adds pyroscope profiling features to DuckDB

> For raw `pprof` generation use the [pprof extension](https://github.com/quackscience/duckdb-extension-pprof)

### Install
```
INSTALL pyroscope FROM community;
LOAD pyroscope;
```

### Usage

```sql
---- Start the tracer, requires backend URL
D SELECT * FROM trace_start('https://pyroscope:4000');

---- Run a bunch of queries to stream results to Pyroscope/qryn

---- Stop the tracer
D SELECT * FROM trace_stop();
```

## Glory Shot in Pyroscope
![image](https://github.com/user-attachments/assets/1992c8b8-dd29-4343-9a54-88363fa5fe8c)
