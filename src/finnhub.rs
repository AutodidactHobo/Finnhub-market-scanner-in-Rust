use serde::Deserialize;
use std::time::Duration;
use crate::config::Config;
use crate::errors::{Result, ScannerError};

#[derive(Debug, Deserialize, Clone)]
pub struct Quote {
    pub c: f64,  // current price
    pub pc: f64, // previous close
    #[serde(default)]
    pub h: f64,  // high
    #[serde(default)]
    pub l: f64,  // low
    #[serde(default)]
    pub o: f64,  // open
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StockQuote {
    pub symbol: String,
    pub price: f64,
    pub prev_close: f64,
    pub change_pct: f64,
    pub high: f64,
    pub low: f64,
    pub open: f64,
}

impl StockQuote {
    pub fn from_quote(symbol: String, quote: Quote) -> Self {
        let change_pct = if quote.pc != 0.0 {
            ((quote.c - quote.pc) / quote.pc) * 100.0
        } else {
            0.0
        };

        Self {
            symbol,
            price: quote.c,
            prev_close: quote.pc,
            change_pct,
            high: quote.h,
            low: quote.l,
            open: quote.o,
        }
    }
}

pub struct FinnhubClient {
    api_key: String,
    client: reqwest::Client,
    config: Config,
}

impl FinnhubClient {
    pub fn new(api_key: String, config: Config) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            api_key,
            client,
            config,
        }
    }

    pub async fn fetch_quote(&self, symbol: &str) -> Result<Quote> {
        let url = format!(
            "https://finnhub.io/api/v1/quote?symbol={}&token={}",
            symbol, self.api_key
        );

        log::debug!("Fetching quote for {}", symbol);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(ScannerError::Api(format!(
                "HTTP {}: {}",
                response.status(),
                symbol
            )));
        }

        let quote: Quote = response.json().await?;

        // Validate we got actual data
        if quote.c == 0.0 && quote.pc == 0.0 {
            return Err(ScannerError::Api(format!("No data for {}", symbol)));
        }

        Ok(quote)
    }

    pub async fn fetch_quotes(&self, symbols: &[String]) -> Result<Vec<StockQuote>> {
        let mut results = Vec::new();
        let mut errors = Vec::new();

        // Process in chunks to respect rate limits
        for chunk in symbols.chunks(self.config.concurrent_requests) {
            let mut tasks = Vec::new();

            for symbol in chunk {
                let client = self.clone();
                let symbol = symbol.clone();

                let task = tokio::spawn(async move {
                    (symbol.clone(), client.fetch_quote(&symbol).await)
                });

                tasks.push(task);
            }

            // Collect results
            for task in tasks {
                match task.await {
                    Ok((symbol, Ok(quote))) => {
                        results.push(StockQuote::from_quote(symbol, quote));
                    }
                    Ok((symbol, Err(e))) => {
                        log::warn!("{}: {}", symbol, e);
                        errors.push(format!("{}: {}", symbol, e));
                    }
                    Err(e) => {
                        log::error!("Task failed: {}", e);
                        errors.push(format!("Task error: {}", e));
                    }
                }
            }

            // Rate limiting between chunks
            tokio::time::sleep(Duration::from_millis(self.config.rate_limit_delay_ms)).await;
        }

        if results.is_empty() && !errors.is_empty() {
            return Err(ScannerError::Api(format!(
                "All requests failed. First error: {}",
                errors[0]
            )));
        }

        if !errors.is_empty() {
            log::info!("Completed with {} errors", errors.len());
        }

        Ok(results)
    }
}

impl Clone for FinnhubClient {
    fn clone(&self) -> Self {
        Self {
            api_key: self.api_key.clone(),
            client: self.client.clone(),
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stock_quote_calculation() {
        let quote = Quote {
            c: 150.0,
            pc: 100.0,
            h: 155.0,
            l: 145.0,
            o: 148.0,
        };

        let stock_quote = StockQuote::from_quote("TEST".to_string(), quote);
        assert_eq!(stock_quote.change_pct, 50.0);
        assert_eq!(stock_quote.price, 150.0);
    }

    #[test]
    fn test_zero_previous_close() {
        let quote = Quote {
            c: 150.0,
            pc: 0.0,
            h: 155.0,
            l: 145.0,
            o: 148.0,
        };

        let stock_quote = StockQuote::from_quote("TEST".to_string(), quote);
        assert_eq!(stock_quote.change_pct, 0.0);
    }
}