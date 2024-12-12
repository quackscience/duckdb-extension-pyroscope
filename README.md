<img src="https://github.com/user-attachments/assets/46a5c546-7e9b-42c7-87f4-bc8defe674e0" width=250 />

# DuckDB Pyroscope Extension
This experimental extension adds pyroscope profiling features to DuckDB

![duckdb_flamegraph](https://github.com/user-attachments/assets/9769ca8c-9839-41d8-8dcc-c468c0637771)

> For raw `pprof` generation use the [pprof extension](https://github.com/quackscience/duckdb-extension-pprof)

### Install
```
INSTALL pyroscope FROM community;
LOAD pyroscope;
```

### Usage

```sql
---- Start the tracer, requires backend Pyroscope URL
D SELECT * FROM trace_start('https://pyroscope:4000');

---- Run a bunch of heavy queries to stream results to Pyroscope/qryn

---- Stop the tracer. This might hang due to a bug in the pyroscope crate.
D SELECT * FROM trace_stop();
```

## Glory Shot in Pyroscope
<!-- ![image](https://github.com/user-attachments/assets/1992c8b8-dd29-4343-9a54-88363fa5fe8c) -->

![image](https://github.com/user-attachments/assets/b58e03e5-576e-42dc-a839-52629fdc0ac8)
