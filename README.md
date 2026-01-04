# Finnhub Market Scanner

A high-performance CLI tool for real-time stock market data analysis. Built in Rust with concurrent API requests, multiple output formats, and advanced filtering.

## Features

- Concurrent request handling with configurable rate limiting
- Multiple output formats: table, JSON, CSV, compact
- Advanced filtering: gainers, losers, minimum change threshold
- Real-time watch mode with auto-refresh
- Flexible configuration via TOML files or environment variables
- Comprehensive error handling and logging
- Cross-platform support: Windows, macOS, Linux

## Requirements

- Rust 1.70 or higher
- Finnhub API key (free at https://finnhub.io)

## Installation

```bash
git clone https://github.com/AutodidactHobo/Finnhub-market-scanner-in-Rust.git
cd Finnhub-market-scanner-in-Rust
cargo build --release
```

## Configuration

Set your API key:

```bash
export FINNHUB_API_KEY=your_api_key_here
```

Optional: Create config.toml

```toml
api_key = "your_api_key_here"
symbols_file = "symbols.txt"
concurrent_requests = 5
rate_limit_delay_ms = 200
timeout_secs = 10
default_output = "table"
```

Optional: Create symbols.txt

```
AAPL
MSFT
GOOGL
TSLA
```

## Usage

Basic scan:
```bash
cargo run --release -- scan --symbols AAPL,MSFT,GOOGL
```

Scan from file:
```bash
cargo run --release -- scan --symbols-file symbols.txt
```

Watch mode (30 second intervals):
```bash
cargo run --release -- watch --symbols AAPL,MSFT --interval 30
```

Filter gainers only:
```bash
cargo run --release -- scan --symbols-file symbols.txt --gainers-only
```

Output as JSON:
```bash
cargo run --release -- scan --symbols AAPL,MSFT --output json
```

Show only significant moves (>2%):
```bash
cargo run --release -- scan --symbols-file symbols.txt --min-change 2.0
```

## Example Output

```
===========================================================================
SYMBOL          PRICE   PREV CLOSE       CHANGE    DAY RANGE
===========================================================================
AAPL           271.01       271.86    -0.31% 269.00-277.84
MSFT           472.94       483.62    -2.21% 470.16-484.66
GOOGL          315.15       313.00 +   0.69% 310.33-322.50
===========================================================================

Summary:
   Total symbols: 3
   Gainers: 1 | Losers: 2
   Average change: -0.61%
   Top gainer: GOOGL (+0.69%)
   Top loser: MSFT (-2.21%)
```

## Command Reference

### scan

Fetch and display stock quotes.

Options:
- -s, --symbols <SYMBOLS>         Comma-separated stock symbols
- -f, --symbols-file <FILE>       File with symbols (one per line)
- -o, --output <FORMAT>           Output format: table, json, csv, compact
- --sort-by-change                Sort by absolute percentage change
- --gainers-only                  Show only positive changes
- --losers-only                   Show only negative changes
- --min-change <PERCENT>          Filter by minimum change threshold

### watch

Monitor stocks with continuous updates.

Options:
- -s, --symbols <SYMBOLS>         Symbols to monitor
- -f, --symbols-file <FILE>       File with symbols
- -i, --interval <SECONDS>        Update interval (default: 60)

### config

Manage configuration.

Options:
- --init                          Initialize default config file
- --show                          Display current configuration

## Architecture

```
src/
├── main.rs      - CLI entry point and argument parsing
├── config.rs    - Configuration management
├── errors.rs    - Error types and handling
├── finnhub.rs   - API client and data fetching
└── output.rs    - Display and formatting logic
```

Technology stack:
- Tokio: Async runtime for concurrent requests
- Clap: CLI argument parsing
- Serde: JSON/TOML serialization
- Reqwest: HTTP client

## Development

Run tests:
```bash
cargo test
```

Format code:
```bash
cargo fmt
```

Lint:
```bash
cargo clippy
```

Debug logging:
```bash
RUST_LOG=debug cargo run -- scan -s AAPL
```

## Performance

Benchmark results on standard hardware:
- 10 stocks: ~1 second
- 100 stocks: ~4 seconds
- 1000 stocks: ~40 seconds

Optimizations:
- Concurrent API requests with configurable batch size
- Memory-efficient with minimal allocations
- Fast startup time under 100ms
- Release builds use LTO and optimized codegen

## License

MIT License

Copyright (c) 2026

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
