use anyhow::Result;
use wasmtime::{Config, Engine};

pub struct WasmEngine {
    engine: Engine,
    config: ExecutionConfig,
}

pub struct ExecutionConfig {
    pub fuel_limit: Option<u64>,
    pub memory_limit: usize,
    pub stdout_limit: usize,
    pub stderr_limit: usize,
    pub epoch_deadline: Option<u64>,
}

impl WasmEngine {
    pub fn new(config: ExecutionConfig) -> Result<Self> {
        let mut wasmtime_config = Config::new();

        if config.fuel_limit.is_some() {
            wasmtime_config.consume_fuel(true);
        }

        if config.epoch_deadline.is_some() {
            wasmtime_config.epoch_interruption(true);
        }

        Ok(Self {
            engine: Engine::new(&wasmtime_config)?,
            config,
        })
    }

    pub fn wasmtime(&self) -> &Engine {
        &self.engine
    }

    pub fn config(&self) -> &ExecutionConfig {
        &self.config
    }
}

impl Default for WasmEngine {
    fn default() -> Self {
        let config = ExecutionConfig {
            fuel_limit: Some(10_000_000),
            memory_limit: 1024 * 1024 * 5,
            stdout_limit: 1024 * 10,
            stderr_limit: 1024 * 10,
            epoch_deadline: None,
        };

        WasmEngine::new(config).expect("failed to create WasmEngine")
    }
}
