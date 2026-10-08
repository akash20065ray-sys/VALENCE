# VALENCE: PROTOCOL SPECIFICATION (RESP3)
> **Document:** `PRD/protocol_spec.md`  
> **Project:** Valence — Next-Generation In-Memory State Engine in Pure Rust  
> **Standard:** Redis Serialization Protocol v3 (RESP3) & v2 Fallback  

---

## 1. Overview & Wire Architecture

Valence communicates with network clients using standard TCP sockets speaking the **RESP3 (REdis Serialization Protocol v3)**. Because Valence implements protocol-level compatibility:
* Existing Redis client libraries (e.g., Python `redis-py`, Node.js `ioredis`, Go `go-redis`, Java `Jedis`) connect to Valence out of the box.
* Official CLI utilities (`redis-cli`, `redis-benchmark`) operate natively against Valence.

All protocol frames use carriage-return line-feed (`\r\n`) as the message delimiter.

---

## 2. Supported Data Framing Types

Valence's zero-copy parser (`src/protocol/resp.rs`) handles the following wire data types:

| Type Byte | RESP3 Data Type | Wire Example | Description |
| :---: | :--- | :--- | :--- |
| `+` | **Simple String** | `+OK\r\n` | Single-line positive status response. |
| `-` | **Simple Error** | `-ERR unknown command 'FOO'\r\n` | Error reply with error type prefix. |
| `:` | **64-bit Integer** | `:1000\r\n` | Signed 64-bit integer response. |
| `$` | **Bulk String** | `$5\r\nhello\r\n` | Binary-safe string with length prefix. |
| `*` | **Array** | `*2\r\n$3\r\nGET\r\n$4\r\nuser\r\n` | Command request array or list response. |
| `_` | **Null** | `_\r\n` | Null response (key not found or expired). |
| `#` | **Boolean** | `#t\r\n` or `#f\r\n` | True/False response. |
| `%` | **Map** | `%2\r\n+key1\r\n:10\r\n+key2\r\n:20\r\n` | Key-value dictionary response. |

---

## 3. Command Syntax & Wire Specifications

### 3.1 `SET` — Insert or Update Key-Value
* **Syntax:** `SET key value [EX seconds] [PX milliseconds] [NX|XX]`
* **Wire Request (Array of Bulk Strings):**
  ```
  *3\r\n$3\r\nSET\r\n$7\r\nuser:42\r\n$5\r\nAkash\r\n
  ```
* **With Expiration (EX):**
  ```
  *5\r\n$3\r\nSET\r\n$7\r\nuser:42\r\n$5\r\nAkash\r\n$2\r\nEX\r\n$2\r\n60\r\n
  ```
* **Engine Semantics:**
  * Without `EX`: Key and value are inserted into the local shard's Robin Hood hash map.
  * With `EX`: Key pointer is inserted into the Robin Hood map and scheduled in the 4-tier Hierarchical Timing Wheel in O(1) time.
* **Response:** `+OK\r\n`

---

### 3.2 `GET` — Retrieve Value by Key
* **Syntax:** `GET key`
* **Wire Request:**
  ```
  *2\r\n$3\r\nGET\r\n$7\r\nuser:42\r\n
  ```
* **Engine Semantics:**
  1. Hash key via MurmurHash3 to identify core shard.
  2. Probe Robin Hood flat array.
  3. If missing: return Null (`_\r\n`).
  4. If tiered to NVMe: issue asynchronous `io_uring` read, await completion, return Bulk String.
  5. If in DRAM: return Bulk String directly: `$5\r\nAkash\r\n`.
* **Response:**
  * Found: `$5\r\nAkash\r\n`
  * Missing / Expired: `_\r\n` (RESP3) or `$-1\r\n` (RESP2 fallback)

---

### 3.3 `DEL` — Delete One or More Keys
* **Syntax:** `DEL key [key ...]`
* **Wire Request:**
  ```
  *2\r\n$3\r\nDEL\r\n$7\r\nuser:42\r\n
  ```
* **Engine Semantics:**
  * Probes key in Robin Hood map.
  * If found, executes backshift deletion: subsequent collided entries shift backward by 1 slot.
  * Detaches any pending timer from the timing wheel.
* **Response:** `:1\r\n` (Integer count of deleted keys).

---

### 3.4 `EXPIRE` — Set Key Time-To-Live
* **Syntax:** `EXPIRE key seconds`
* **Wire Request:**
  ```
  *3\r\n$6\r\nEXPIRE\r\n$7\r\nuser:42\r\n$2\r\n60\r\n
  ```
* **Response:**
  * Key exists and timer set: `:1\r\n`
  * Key does not exist: `:0\r\n`

---

### 3.5 `TTL` — Query Key Remaining Lifetime
* **Syntax:** `TTL key`
* **Wire Request:**
  ```
  *2\r\n$3\r\nTTL\r\n$7\r\nuser:42\r\n
  ```
* **Response:**
  * Integer remaining seconds (e.g., `:42\r\n`).
  * If key exists without TTL: `:-1\r\n`.
  * If key does not exist or expired: `:-2\r\n`.

---

### 3.6 `PING` — Connection Health Check
* **Syntax:** `PING [message]`
* **Response:** `+PONG\r\n` or `$len\r\nmessage\r\n`

---

### 3.7 `EXPLAIN` — Diagnostic Inspection (Valence Native)
* **Syntax:** `EXPLAIN key`
* **Purpose:** Valence-specific diagnostic command used by `valence-cli` and performance engineers.
* **Wire Response (RESP3 Map `%`):**
  ```
  %6\r\n
  +shard_id\r\n:3\r\n
  +core_affinity\r\n+CPU-3\r\n
  +probe_depth\r\n:1\r\n
  +storage_tier\r\n+DRAM_HOT\r\n
  +ttl_status\r\n+TIER_2_VALID\r\n
  +execution_latency_ns\r\n:180\r\n
  ```

---

## 4. Connection State Machine

```
   [ Client Connects ]
            │
            ▼
   [ Stream Handshake ] ──(RESP3 Hello / Inline)──► [ Set Protocol Mode ]
            │
            ▼
   [ Command Loop ] ◄─────────────────────────────────────────┐
            │                                                 │
            ├──► Read TCP Buffer into Ring Buffer             │
            ├──► SIMD AVX2 Scanner checks for \r\n            │
            ├──► Parse Array Frames                           │
            ├──► Route Command to Target Core Shard           │
            ├──► Execute bounded logic (<1µs)                 │
            ├──► Serialize Reply Buffer                       │
            └──► Flush bytes to Client Socket ────────────────┘
```

---

## 5. Error Frame Specifications

When a command violates constraints, Valence returns standard structured RESP error frames:

| Error Case | Wire Response |
| :--- | :--- |
| **Unknown Command** | `-ERR unknown command 'XYZ'\r\n` |
| **Wrong Argument Count** | `-ERR wrong number of arguments for 'set' command\r\n` |
| **Out of Memory** | `-OOM command not allowed when used memory > maxmemory\r\n` |
| **Syntax Error** | `-ERR syntax error\r\n` |
| **Invalid Integer / Float** | `-ERR value is not an integer or out of range\r\n` |
