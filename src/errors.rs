use std::fmt;

pub type Result<T> = std::result::Result<T, ScannerError>;

#[derive(Debug)]
pub enum ScannerError {
    /// Configuration errors
    Config(String),
    
    /// API errors
    Api(String),
    
    /// Network errors
    Network(String),
    
    /// I/O errors
    Io(String),
    
    /// Data parsing errors
    Parse(String),
    
    /// No symbols provided
    NoSymbols,
    
    /// Invalid input
    InvalidInput(String),
}

impl fmt::Display for ScannerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScannerError::Config(msg) => write!(f, "Configuration error: {}", msg),
            ScannerError::Api(msg) => write!(f, "API error: {}", msg),
            ScannerError::Network(msg) => write!(f, "Network error: {}", msg),
            ScannerError::Io(msg) => write!(f, "I/O error: {}", msg),
            ScannerError::Parse(msg) => write!(f, "Parse error: {}", msg),
            ScannerError::NoSymbols => write!(f, "No symbols provided. Use --symbols, --symbols-file, or configure symbols_file in config"),
            ScannerError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for ScannerError {}

// Convert from reqwest errors
impl From<reqwest::Error> for ScannerError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ScannerError::Network(format!("Request timeout: {}", err))
        } else if err.is_connect() {
            ScannerError::Network(format!("Connection failed: {}", err))
        } else if err.is_status() {
            ScannerError::Api(format!("HTTP error: {}", err))
        } else {
            ScannerError::Network(format!("Request failed: {}", err))
        }
    }
}

// Convert from I/O errors
impl From<std::io::Error> for ScannerError {
    fn from(err: std::io::Error) -> Self {
        ScannerError::Io(err.to_string())
    }
}

// Convert from serde_json errors
impl From<serde_json::Error> for ScannerError {
    fn from(err: serde_json::Error) -> Self {
        ScannerError::Parse(format!("JSON parsing failed: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ScannerError::Config("test error".to_string());
        assert_eq!(err.to_string(), "Configuration error: test error");
    }

    #[test]
    fn test_no_symbols_error() {
        let err = ScannerError::NoSymbols;
        assert!(err.to_string().contains("No symbols provided"));
    }
}