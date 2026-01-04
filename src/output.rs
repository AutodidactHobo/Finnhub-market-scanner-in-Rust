use clap::ValueEnum;
use serde::Serialize;
use std::io::{self, Write};
use crate::errors::Result;
use crate::finnhub::StockQuote;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    /// Pretty table format
    Table,
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// Compact format
    Compact,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Table
    }
}

pub fn display(quotes: &[StockQuote], format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Table => {
            display_table(quotes);
            Ok(())
        }
        OutputFormat::Json => display_json(quotes),
        OutputFormat::Csv => display_csv(quotes),
        OutputFormat::Compact => {
            display_compact(quotes);
            Ok(())
        }
    }
}

fn display_table(quotes: &[StockQuote]) {
    println!("\n{}", "=".repeat(75));
    println!(
        "{:<8} {:>12} {:>12} {:>12} {:>12}",
        "SYMBOL", "PRICE", "PREV CLOSE", "CHANGE", "DAY RANGE"
    );
    println!("{}", "=".repeat(75));

    for quote in quotes {
        let range = if quote.high > 0.0 && quote.low > 0.0 {
            format!("{:.2}-{:.2}", quote.low, quote.high)
        } else {
            "N/A".to_string()
        };

        println!(
            "{:<8} {:>12.2} {:>12.2} {} {:>12}",
            quote.symbol,
            quote.price,
            quote.prev_close,
            format_change(quote.change_pct),
            range
        );
    }

    println!("{}", "=".repeat(75));
    display_summary(quotes);
}

fn display_json(quotes: &[StockQuote]) -> Result<()> {
    #[derive(Serialize)]
    struct JsonOutput<'a> {
        quotes: &'a [StockQuote],
        summary: Summary,
    }

    let summary = calculate_summary(quotes);
    let output = JsonOutput { quotes, summary };
    
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn display_csv(quotes: &[StockQuote]) -> Result<()> {
    println!("symbol,price,prev_close,change_pct,high,low,open");
    for quote in quotes {
        println!(
            "{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}",
            quote.symbol,
            quote.price,
            quote.prev_close,
            quote.change_pct,
            quote.high,
            quote.low,
            quote.open
        );
    }
    Ok(())
}

fn display_compact(quotes: &[StockQuote]) {
    for quote in quotes {
        let arrow = if quote.change_pct > 0.0 {
            "↑"
        } else if quote.change_pct < 0.0 {
            "↓"
        } else {
            "→"
        };

        println!(
            "{:<6} ${:>8.2} {} {}",
            quote.symbol,
            quote.price,
            arrow,
            format_change(quote.change_pct)
        );
    }
}

fn format_change(change_pct: f64) -> String {
    if change_pct > 0.0 {
        format!("\x1b[32m+{:>7.2}%\x1b[0m", change_pct)
    } else if change_pct < 0.0 {
        format!("\x1b[31m{:>8.2}%\x1b[0m", change_pct)
    } else {
        format!("{:>8.2}%", change_pct)
    }
}

#[derive(Serialize)]
struct Summary {
    total: usize,
    gainers: usize,
    losers: usize,
    avg_change: f64,
    top_gainer: Option<TopStock>,
    top_loser: Option<TopStock>,
}

#[derive(Serialize)]
struct TopStock {
    symbol: String,
    change_pct: f64,
}

fn calculate_summary(quotes: &[StockQuote]) -> Summary {
    let total = quotes.len();
    let gainers = quotes.iter().filter(|q| q.change_pct > 0.0).count();
    let losers = quotes.iter().filter(|q| q.change_pct < 0.0).count();
    
    let avg_change = if total > 0 {
        quotes.iter().map(|q| q.change_pct).sum::<f64>() / total as f64
    } else {
        0.0
    };

    let top_gainer = quotes
        .iter()
        .max_by(|a, b| a.change_pct.partial_cmp(&b.change_pct).unwrap())
        .map(|q| TopStock {
            symbol: q.symbol.clone(),
            change_pct: q.change_pct,
        });

    let top_loser = quotes
        .iter()
        .min_by(|a, b| a.change_pct.partial_cmp(&b.change_pct).unwrap())
        .map(|q| TopStock {
            symbol: q.symbol.clone(),
            change_pct: q.change_pct,
        });

    Summary {
        total,
        gainers,
        losers,
        avg_change,
        top_gainer,
        top_loser,
    }
}

fn display_summary(quotes: &[StockQuote]) {
    if quotes.is_empty() {
        return;
    }

    let summary = calculate_summary(quotes);

    println!("\n📈 Summary:");
    println!("   Total symbols: {}", summary.total);
    println!(
        "   Gainers: \x1b[32m{}\x1b[0m | Losers: \x1b[31m{}\x1b[0m",
        summary.gainers, summary.losers
    );
    println!("   Average change: {}", format_change(summary.avg_change));

    if let Some(top) = summary.top_gainer {
        println!("   Top gainer: {} ({})", top.symbol, format_change(top.change_pct));
    }

    if let Some(top) = summary.top_loser {
        println!("   Top loser: {} ({})", top.symbol, format_change(top.change_pct));
    }

    println!();
}

pub fn filter_quotes(
    quotes: Vec<StockQuote>,
    gainers_only: bool,
    losers_only: bool,
    min_change: Option<f64>,
) -> Vec<StockQuote> {
    quotes
        .into_iter()
        .filter(|q| {
            if gainers_only && q.change_pct <= 0.0 {
                return false;
            }
            if losers_only && q.change_pct >= 0.0 {
                return false;
            }
            if let Some(min) = min_change {
                if q.change_pct.abs() < min {
                    return false;
                }
            }
            true
        })
        .collect()
}

pub fn sort_by_change(mut quotes: Vec<StockQuote>) -> Vec<StockQuote> {
    quotes.sort_by(|a, b| {
        b.change_pct
            .abs()
            .partial_cmp(&a.change_pct.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    quotes
}

pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finnhub::StockQuote;

    fn create_test_quote(symbol: &str, change_pct: f64) -> StockQuote {
        StockQuote {
            symbol: symbol.to_string(),
            price: 100.0,
            prev_close: 100.0 - change_pct,
            change_pct,
            high: 105.0,
            low: 95.0,
            open: 98.0,
        }
    }

    #[test]
    fn test_filter_gainers_only() {
        let quotes = vec![
            create_test_quote("GAIN", 5.0),
            create_test_quote("LOSS", -3.0),
            create_test_quote("FLAT", 0.0),
        ];

        let filtered = filter_quotes(quotes, true, false, None);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].symbol, "GAIN");
    }

    #[test]
    fn test_filter_losers_only() {
        let quotes = vec![
            create_test_quote("GAIN", 5.0),
            create_test_quote("LOSS", -3.0),
            create_test_quote("FLAT", 0.0),
        ];

        let filtered = filter_quotes(quotes, false, true, None);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].symbol, "LOSS");
    }

    #[test]
    fn test_min_change_filter() {
        let quotes = vec![
            create_test_quote("BIG", 10.0),
            create_test_quote("SMALL", 1.0),
            create_test_quote("NEG", -5.0),
        ];

        let filtered = filter_quotes(quotes, false, false, Some(3.0));
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_sort_by_change() {
        let quotes = vec![
            create_test_quote("A", 2.0),
            create_test_quote("B", -5.0),
            create_test_quote("C", 10.0),
        ];

        let sorted = sort_by_change(quotes);
        assert_eq!(sorted[0].symbol, "C"); // 10%
        assert_eq!(sorted[1].symbol, "B"); // -5%
        assert_eq!(sorted[2].symbol, "A"); // 2%
    }
}