# Crabex 🦀

Crabex is a blazing-fast, purely in-memory Centralized Exchange (CEX) ecosystem built entirely in Rust. It implements strict **Price-Time Priority** matching logic and operates across a multi-binary microservice architecture.

## Architecture

This project is built as a Cargo Workspace containing four interconnected services:

1. **`engine` (The Core):** A high-performance matching engine using `BTreeMap` and `VecDeque` for zero-allocation, ultra-low latency order matching. Runs an asynchronous `tokio` TCP server.
2. **`tui` (The Dashboard):** A real-time terminal user interface built with `ratatui`. Connects to the engine via TCP to stream the live order book and trade history.
3. **`bot-mm` (Market Maker):** An automated liquidity provider that continuously quotes bid/ask spreads to keep the engine's order book populated.
4. **`bot-arb` (Arbitrageur):** A sniper bot that monitors the engine for pricing inefficiencies and executes split-second market orders to capture arbitrage profit.

## Tech Stack

*   **Language:** Rust (Strictly typed, memory-safe, zero-cost abstractions)
*   **Async Runtime:** `tokio`
*   **Terminal UI:** `ratatui` + `crossterm`
*   **Serialization:** `serde` + `serde_json`

## Getting Started

### Prerequisites
Make sure you have Rust and Cargo installed.

### Running the Ecosystem
Because this is a microservice architecture, you will need to run the binaries in separate terminal windows.

**1. Start the Matching Engine:**
```bash
cargo run -p engine