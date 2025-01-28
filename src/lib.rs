extern crate duckdb;
extern crate duckdb_loadable_macros;
extern crate libduckdb_sys;
extern crate pyroscope;
extern crate pyroscope_pprofrs;

use duckdb::{
    core::{DataChunkHandle, Inserter, LogicalTypeHandle, LogicalTypeId},
    vtab::{BindInfo, Free, FunctionInfo, InitInfo, VTab},
    Connection, Result,
};
use duckdb_loadable_macros::duckdb_entrypoint_c_api;
use libduckdb_sys as ffi;
use std::{
    error::Error,
    ffi::{c_char, CString},
    sync::Arc,
    sync::Mutex,
};
use pyroscope::pyroscope::PyroscopeAgentRunning;
use pyroscope::PyroscopeAgent;
use pyroscope_pprofrs::{pprof_backend, PprofConfig};

// Store just the running agent
lazy_static::lazy_static! {
    static ref PYROSCOPE_AGENT: Arc<Mutex<Option<PyroscopeAgent<PyroscopeAgentRunning>>>> = Arc::new(Mutex::new(None));
}

// Empty struct that implements Free for BindData
#[repr(C)]
struct EmptyBindData;

impl Free for EmptyBindData {}

// Trace Start implementation
struct TraceStartVTab;

#[repr(C)]
struct TraceStartBindData {
    url: *mut c_char,
}

#[repr(C)]
struct TraceStartInitData {
    done: bool,
}

impl Free for TraceStartBindData {
    fn free(&mut self) {
        unsafe {
            if !self.url.is_null() {
                drop(CString::from_raw(self.url));
            }
        }
    }
}

impl Free for TraceStartInitData {}

impl VTab for TraceStartVTab {
    type InitData = TraceStartInitData;
    type BindData = TraceStartBindData;

    unsafe fn bind(bind: &BindInfo, data: *mut TraceStartBindData) -> Result<(), Box<dyn Error>> {
        bind.add_result_column("status", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        let url = bind.get_parameter(0).to_string();
        unsafe {
            (*data).url = CString::new(url).unwrap().into_raw();
        }
        Ok(())
    }

    unsafe fn init(_: &InitInfo, data: *mut TraceStartInitData) -> Result<(), Box<dyn Error>> {
        unsafe {
            (*data).done = false;
        }
        Ok(())
    }

    unsafe fn func(func: &FunctionInfo, output: &mut DataChunkHandle) -> Result<(), Box<dyn Error>> {
        let init_info = func.get_init_data::<TraceStartInitData>();
        let bind_info = func.get_bind_data::<TraceStartBindData>();
        
        unsafe {
            if (*init_info).done {
                output.set_len(0);
                return Ok(());
            }
            
            (*init_info).done = true;
            
            let url_cstr = CString::from_raw((*bind_info).url);
            let url_str = url_cstr.to_str()?;
            
            let mut agent_lock = PYROSCOPE_AGENT.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
            if agent_lock.is_some() {
                let vector = output.flat_vector(0);
                vector.insert(0, CString::new("Profiling already running")?);
                output.set_len(1);
                return Ok(());
            }

            // Create and start the agent
            let agent = PyroscopeAgent::builder(url_str, "duckdb-profile")
                .backend(pprof_backend(PprofConfig::new().sample_rate(100)))
                .build()
                .map_err(|e| format!("Failed to create Pyroscope agent: {}", e))?;

            let running_agent = agent.start()
                .map_err(|e| format!("Failed to start Pyroscope agent: {}", e))?;
            
            *agent_lock = Some(running_agent);
            
            let vector = output.flat_vector(0);
            vector.insert(0, CString::new("Profiling started with Pyroscope")?);
            output.set_len(1);
            
            (*bind_info).url = CString::into_raw(url_cstr);
        }
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
    done: bool,
}

impl Free for TraceStopInitData {}

impl VTab for TraceStopVTab {
    type InitData = TraceStopInitData;
    type BindData = EmptyBindData;

    unsafe fn bind(bind: &BindInfo, _: *mut EmptyBindData) -> Result<(), Box<dyn Error>> {
        bind.add_result_column("status", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        Ok(())
    }

    unsafe fn init(_: &InitInfo, data: *mut TraceStopInitData) -> Result<(), Box<dyn Error>> {
        unsafe {
            (*data).done = false;
        }
        Ok(())
    }

    unsafe fn func(func: &FunctionInfo, output: &mut DataChunkHandle) -> Result<(), Box<dyn Error>> {
        let init_info = func.get_init_data::<TraceStopInitData>();
        
        unsafe {
            if (*init_info).done {
                output.set_len(0);
                return Ok(());
            }
            
            (*init_info).done = true;
            
            let mut agent_lock = PYROSCOPE_AGENT.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
            
            if let Some(running_agent) = agent_lock.take() {
                // shutdown() returns (), so we just call it directly
                running_agent.shutdown();
                
                let vector = output.flat_vector(0);
                vector.insert(0, CString::new("Profiling stopped successfully")?);
                output.set_len(1);
            } else {
                let vector = output.flat_vector(0);
                vector.insert(0, CString::new("No profiling session running")?);
                output.set_len(1);
            }
        }
        Ok(())
    }

    fn parameters() -> Option<Vec<LogicalTypeHandle>> {
        None
    }
}

#[duckdb_entrypoint_c_api(ext_name = "pyroscope", min_duckdb_version = "v1.2.0")]
pub unsafe fn extension_entrypoint(con: Connection) -> Result<(), Box<dyn Error>> {
    con.register_table_function::<TraceStartVTab>("trace_start")
        .expect("Failed to register trace_start function");
    con.register_table_function::<TraceStopVTab>("trace_stop")
        .expect("Failed to register trace_stop function");
    Ok(())
}
