use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use crate::errors::{Result, ScannerError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Finnhub API key
    pub api_key: String,
    
    /// Optional path to symbols file
    pub symbols_file: Option<PathBuf>,
    
    /// Number of concurrent requests
    #[serde(default = "default_concurrent_requests")]
    pub concurrent_requests: usize,
    
    /// Rate limit delay in milliseconds
    #[serde(default = "default_rate_limit_delay")]
    pub rate_limit_delay_ms: u64,
    
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    
    /// Default output format
    #[serde(default)]
    pub default_output: String,
}

fn default_concurrent_requests() -> usize {
    5
}

fn default_rate_limit_delay() -> u64 {
    200
}

fn default_timeout() -> u64 {
    10
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::from("YOUR_API_KEY_HERE"),
            symbols_file: Some(PathBuf::from("symbols.txt")),
            concurrent_requests: default_concurrent_requests(),
            rate_limit_delay_ms: default_rate_limit_delay(),
            timeout_secs: default_timeout(),
            default_output: String::from("table"),
        }
    }
}

impl Config {
    /// Load config from TOML file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| ScannerError::Config(format!("Failed to read config file: {}", e)))?;
        
        let config: Config = toml::from_str(&content)
            .map_err(|e| ScannerError::Config(format!("Failed to parse config: {}", e)))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// Load config from environment variables or use defaults
    pub fn from_env_or_default() -> Result<Self> {
        let mut config = Config::default();
        
        // Check for API key in environment
        if let Ok(key) = std::env::var("FINNHUB_API_KEY") {
            config.api_key = key;
        }
        
        // Check for symbols file in environment
        if let Ok(file) = std::env::var("SYMBOLS_FILE") {
            config.symbols_file = Some(PathBuf::from(file));
        }
        
        config.validate()?;
        Ok(config)
    }
    
    /// Save config to TOML file
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| ScannerError::Config(format!("Failed to serialize config: {}", e)))?;
        
        fs::write(path, content)
            .map_err(|e| ScannerError::Config(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }
    
    /// Validate configuration
    fn validate(&self) -> Result<()> {
        if self.api_key.is_empty() || self.api_key == "YOUR_API_KEY_HERE" {
            return Err(ScannerError::Config(
                "API key not configured. Set FINNHUB_API_KEY environment variable or update config file".to_string()
            ));
        }
        
        if self.concurrent_requests == 0 {
            return Err(ScannerError::Config(
                "concurrent_requests must be greater than 0".to_string()
            ));
        }
        
        Ok(())
    }
}

/// Load symbols from a text file (one per line)
pub fn load_symbols_from_file(path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(path)
        .map_err(|e| ScannerError::Io(format!("Failed to read symbols file: {}", e)))?;
    
    let symbols: Vec<String> = content
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .map(|s| s.to_uppercase())
        .collect();
    
    if symbols.is_empty() {
        return Err(ScannerError::NoSymbols);
    }
    
    Ok(symbols)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.concurrent_requests, 5);
        assert_eq!(config.rate_limit_delay_ms, 200);
    }

    #[test]
    fn test_load_symbols_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "AAPL").unwrap();
        writeln!(file, "# Comment").unwrap();
        writeln!(file, "msft").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "GOOGL").unwrap();
        
        let symbols = load_symbols_from_file(file.path()).unwrap();
        assert_eq!(symbols, vec!["AAPL", "MSFT", "GOOGL"]);
    }

    #[test]
    fn test_empty_symbols_file() {
        let file = NamedTempFile::new().unwrap();
        let result = load_symbols_from_file(file.path());
        assert!(result.is_err());
    }
}