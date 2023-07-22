use std::{borrow::BorrowMut, path::PathBuf, sync::Once};

use clap::{Parser, ValueEnum};
use figment::providers::Format;
use serde::Deserialize;

static mut CONFIG: Option<BalancerConfig> = None;

static CONFIG_INIT: Once = Once::new();

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct BalancerConfig {
    /// The port to listen on for HTTP requests.
    pub port: u16,
}

impl Default for BalancerConfig {
    fn default() -> Self {
        Self { port: 8081 }
    }
}

impl BalancerConfig {
    pub fn load(path: &PathBuf) -> anyhow::Result<()> {
        let config = figment::Figment::new()
            .merge(figment::providers::Toml::file(path))
            .merge(figment::providers::Env::prefixed("BALANCER_"))
            .extract()?;
        // SAFETY: CONFIG is only mutated once, and only from this thread. All other accesses are read-only.
        CONFIG_INIT.call_once(|| unsafe { *CONFIG.borrow_mut() = Some(config) });
        Ok(())
    }

    pub fn get() -> &'static Self {
        // SAFETY: get is never called before CONFIG is initialized.
        unsafe { CONFIG.as_ref().expect("config not initialized") }
    }
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(short, long, default_value = "balancer.toml")]
    pub config_path: PathBuf,

    #[clap(short, long, default_value_t = LogLevel::Info, value_enum)]
    pub log_level: LogLevel,

    /// Enable debug logging so that tokio-console works.
    #[clap(long)]
    pub debug_runtime: bool,
}

impl Cli {
    pub fn build_tracing_filter(&self) -> String {
        let mut filter: String = self.log_level.into();
        if self.debug_runtime {
            filter.push_str(",tokio=trace,runtime=trace");
        }
        filter
    }
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[clap(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl From<LogLevel> for String {
    fn from(val: LogLevel) -> Self {
        match val {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
        .into()
    }
}

impl From<LogLevel> for tracing::Level {
    fn from(val: LogLevel) -> Self {
        match val {
            LogLevel::Trace => Self::TRACE,
            LogLevel::Debug => Self::DEBUG,
            LogLevel::Info => Self::INFO,
            LogLevel::Warn => Self::WARN,
            LogLevel::Error => Self::ERROR,
        }
    }
}