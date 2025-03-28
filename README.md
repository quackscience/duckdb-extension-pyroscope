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

### Glory Shot in Pyroscope
Create a `Free` account on [Grafana Cloud](https://grafana.com/auth/sign-up/create-user?pg=prod-cloud&plcmt=hero-btn-1) create a Token for Pyroscope profile sending and use the extension:
```sql
---- Start the tracer to Grafana Cloud Pyroscope
D SELECT * FROM trace_start('https://user:token@profiles-prod-xxx.grafana.net');
```


<!-- ![image](https://github.com/user-attachments/assets/1992c8b8-dd29-4343-9a54-88363fa5fe8c) -->

![pyroscope_duckdb_large](https://github.com/user-attachments/assets/74fad3ec-3bc3-4880-be4b-8149c5431115)
