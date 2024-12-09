# DuckDB Pyroscope Extension

### Build
```
make configure
make debug
```

### Test

```sql
D LOAD './build/debug/pyroscope.duckdb_extension';
---- Start the tracer, requires backend URL
D SELECT * FROM trace_start('https://pyroscope:4000');
---- Stop the tracer
D SELECT * FROM trace_stop();
```
