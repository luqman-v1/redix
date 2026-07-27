# Redix Advanced Features Guide

Redix goes beyond basic key visualization to provide a suite of powerful, enterprise-grade tools designed specifically to debug, monitor, and manage Redis servers.

## 📡 Live Pub/Sub Viewer
Located in the sidebar, the **Pub/Sub Viewer** allows you to subscribe to Redis channels and watch messages arrive in real-time.

* **How to use:** Click the `📡 Pub/Sub` button on the sidebar. Type the channel name you wish to listen to, and click `Subscribe`.
* **Features:**
  * Supports subscribing to multiple channels simultaneously.
  * Captures real-time messages with timestamps.
  * Formats incoming JSON payloads automatically for easy readability.

---

## 🧠 Redis Memory Analyzer (Top Keys Scanner)
A high-performance "X-Ray" tool to identify memory leaks and oversized keys that are draining your server's RAM.

* **How to use:** Click the `🧠 Memory Analyzer` (Radar) icon on the sidebar.
* **Features:**
  * **Chunked Concurrent Scanning:** Powered by Rust, the scanner fires `MEMORY USAGE <key>` commands in parallel chunks (up to 500 keys at a time) ensuring blazing fast analysis without blocking or crashing the Redis server.
  * **Adjustable Sample Size:** Scan a small subset (1,000 keys) or deep-scan up to 100,000 keys.
  * **Interactive Table:** The results are rendered in a sleek table, sorted by memory footprint. Keys are color-coded by type, and you can jump straight to editing a key by clicking the arrow next to it.
  * **Human-Readable Formats:** Translates bytes into readable KB, MB, or GB automatically.

---

## 🐌 Slow Log Profiler
When your Redis server experiences latency spikes, the **Slow Log Profiler** helps you identify the exact commands responsible.

* **How to use:** Click the `🐌 Slow Log` (Snail/Magnifying Glass) icon on the sidebar.
* **Features:**
  * Connects natively to the Redis `SLOWLOG GET` command.
  * Displays execution timestamps and execution duration in microseconds (`µs`) and milliseconds (`ms`).
  * **Highlighting:** Queries exceeding a dangerous threshold (e.g., > 50ms) are highlighted in bright red.
  * Displays the exact command and arguments that caused the bottleneck, as well as the Client IP and Port if available.
  * Includes a button to safely `Reset` the slow logs from the server.

---

## 💾 Two-Way Data Import & Export (CSV)
Easily migrate data in and out of your complex data structures.

* **Export:** Click `Export CSV` on any Hash, List, Set, or Sorted Set to generate an instantly downloadable `.csv` file.
* **Import:** Click `Import CSV` to upload data directly into the current key.
  * **Smart Headers:** The tool automatically detects if your CSV has a header row (like `Field,Value` or `Score,Member`) and safely ignores it during import.
  * **Batch Engine:** Data is imported using parallel chunking (via `Promise.all` batches of 50) directly through the Rust backend. This ensures thousands of rows can be imported instantly without lagging the user interface.

---

## ⏱️ Live TTL Monitoring
Unlike older tools where Time-To-Live (TTL) is a static number, Redix provides a real-time countdown.

* **Global Ticker:** The sidebar tree actively ticks down the TTL of your expiring keys without needing to repeatedly query the Redis server.
* **Inline Editing:** Click any TTL badge to instantly change or remove the expiration time.
