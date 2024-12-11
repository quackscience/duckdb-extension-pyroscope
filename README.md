<img src="https://github.com/user-attachments/assets/46a5c546-7e9b-42c7-87f4-bc8defe674e0" width=250 />

# DuckDB Pyroscope Extension
This experimental extension adds pyroscope profiling features to DuckDB

### Build
```
make configure
make debug
```

### Test

```sql
---- Load the Extension
D LOAD './build/debug/pyroscope.duckdb_extension';

---- Start the tracer, requires backend URL
D SELECT * FROM trace_start('https://pyroscope:4000');

---- Stop the tracer
D SELECT * FROM trace_stop();
```

## Glory Shot in Pyroscope
![image](https://github.com/user-attachments/assets/1992c8b8-dd29-4343-9a54-88363fa5fe8c)
