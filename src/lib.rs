extern crate duckdb;
extern crate duckdb_loadable_macros;
extern crate libduckdb_sys;
extern crate pyroscope;
extern crate pyroscope_pprofrs;

use duckdb::{
    core::{DataChunkHandle, Inserter, LogicalTypeHandle, LogicalTypeId},
    vtab::{BindInfo, InitInfo, TableFunctionInfo, VTab},
    Connection, Result,
};
use duckdb_loadable_macros::duckdb_entrypoint_c_api;
use libduckdb_sys as ffi;
use std::{
    error::Error,
    ffi::CString,
    sync::Arc,
    sync::Mutex,
    sync::atomic::{AtomicBool, Ordering},
};
use pyroscope::pyroscope::PyroscopeAgentRunning;
use pyroscope::PyroscopeAgent;
use pyroscope_pprofrs::{pprof_backend, PprofConfig};

// Store just the running agent
lazy_static::lazy_static! {
    static ref PYROSCOPE_AGENT: Arc<Mutex<Option<PyroscopeAgent<PyroscopeAgentRunning>>>> = Arc::new(Mutex::new(None));
}

// Trace Start implementation
struct TraceStartVTab;

#[repr(C)]
struct TraceStartBindData {
    url: String,
}

#[repr(C)]
struct TraceStartInitData {
    done: AtomicBool,
}

impl VTab for TraceStartVTab {
    type InitData = TraceStartInitData;
    type BindData = TraceStartBindData;

    fn bind(bind: &BindInfo) -> Result<Self::BindData, Box<dyn Error>> {
        bind.add_result_column("status", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        let url = bind.get_parameter(0).to_string();
        Ok(TraceStartBindData { url })
    }

    fn init(_: &InitInfo) -> Result<Self::InitData, Box<dyn Error>> {
        Ok(TraceStartInitData {
            done: AtomicBool::new(false)
        })
    }

    fn func(func: &TableFunctionInfo<Self>, output: &mut DataChunkHandle) -> Result<(), Box<dyn Error>> {
        let init_data = func.get_init_data();
        let bind_data = func.get_bind_data();
        
        if init_data.done.swap(true, Ordering::Relaxed) {
            output.set_len(0);
            return Ok(());
        }
        
        let mut agent_lock = PYROSCOPE_AGENT.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
        if agent_lock.is_some() {
            let vector = output.flat_vector(0);
            vector.insert(0, CString::new("Profiling already running")?);
            output.set_len(1);
            return Ok(());
        }

        // Create and start the agent
        let agent = PyroscopeAgent::builder(bind_data.url.as_str(), "duckdb-profile")
            .backend(pprof_backend(PprofConfig::new().sample_rate(100)))
            .build()
            .map_err(|e| format!("Failed to create Pyroscope agent: {}", e))?;

        let running_agent = agent.start()
            .map_err(|e| format!("Failed to start Pyroscope agent: {}", e))?;
        
        *agent_lock = Some(running_agent);
        
        let vector = output.flat_vector(0);
        vector.insert(0, CString::new("Profiling started with Pyroscope")?);
        output.set_len(1);
        
        Ok(())
    }

    fn parameters() -> Option<Vec<LogicalTypeHandle>> {
        Some(vec![LogicalTypeHandle::from(LogicalTypeId::Varchar)])
    }
}

// Trace Stop implementation
struct TraceStopVTab;

#[repr(C)]
struct TraceStopInitData {
    done: AtomicBool,
}

#[repr(C)]
struct EmptyBindData;

impl VTab for TraceStopVTab {
    type InitData = TraceStopInitData;
    type BindData = EmptyBindData;

    fn bind(bind: &BindInfo) -> Result<Self::BindData, Box<dyn Error>> {
        bind.add_result_column("status", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        Ok(EmptyBindData)
    }

    fn init(_: &InitInfo) -> Result<Self::InitData, Box<dyn Error>> {
        Ok(TraceStopInitData {
            done: AtomicBool::new(false)
        })
    }

    fn func(func: &TableFunctionInfo<Self>, output: &mut DataChunkHandle) -> Result<(), Box<dyn Error>> {
        let init_data = func.get_init_data();
        
        if init_data.done.swap(true, Ordering::Relaxed) {
            output.set_len(0);
            return Ok(());
        }
        
        let mut agent_lock = PYROSCOPE_AGENT.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
        
        if let Some(running_agent) = agent_lock.take() {
            running_agent.shutdown();
            
            let vector = output.flat_vector(0);
            vector.insert(0, CString::new("Profiling stopped successfully")?);
            output.set_len(1);
        } else {
            let vector = output.flat_vector(0);
            vector.insert(0, CString::new("No profiling session running")?);
            output.set_len(1);
        }
        
        Ok(())
    }

    fn parameters() -> Option<Vec<LogicalTypeHandle>> {
        None
    }
}

const EXTENSION_NAME: &str = env!("CARGO_PKG_NAME");

#[duckdb_entrypoint_c_api()]
pub unsafe fn extension_entrypoint(con: Connection) -> Result<(), Box<dyn Error>> {
    con.register_table_function::<TraceStartVTab>("trace_start")
        .expect("Failed to register trace_start function");
    con.register_table_function::<TraceStopVTab>("trace_stop")
        .expect("Failed to register trace_stop function");
    Ok(())
}
