# CrypRQ Core FFI

The `cryp-rq-core` crate exposes a stable C ABI for mobile and desktop hosts. The API is intentionally minimal while the data-plane matures. The current surface covers lifecycle control; packet send/receive will be enabled once the tunnel implementation is production ready.

## Headers

Generate headers with [cbindgen](https://github.com/mozilla/cbindgen):

```bash
cargo install cbindgen
cbindgen --config cbindgen.toml --crate cryprq_core --output cryprq_core.h
```

## Types

```c
typedef struct {
    const char *log_level;         // Optional (NULL = default info)
    const CrypRqStrView *allow_peers;
    size_t allow_peers_len;
} CrypRqConfig;

typedef enum {
    CRYPRQ_CONNECTION_MODE_LISTEN = 0,
    CRYPRQ_CONNECTION_MODE_DIAL = 1,
} CrypRqConnectionMode;

typedef struct {
    CrypRqConnectionMode mode;
    const char *multiaddr;
} CrypRqPeerParams;

typedef enum {
    CRYPRQ_OK = 0,
    CRYPRQ_ERR_NULL = 1,
    CRYPRQ_ERR_UTF8 = 2,
    CRYPRQ_ERR_INVALID_ARGUMENT = 3,
    CRYPRQ_ERR_ALREADY_CONNECTED = 4,
    CRYPRQ_ERR_UNSUPPORTED = 5,
    CRYPRQ_ERR_RUNTIME = 6,
    CRYPRQ_ERR_INTERNAL = 255,
} CrypRqErrorCode;
```

`CrypRqStrView` is a borrowed UTF-8 slice:

```c
typedef struct {
    const char *data;
    size_t len;
} CrypRqStrView;
```

## Functions

```c
CrypRqErrorCode cryprq_init(const CrypRqConfig *config, CrypRqHandle **out_handle);
CrypRqErrorCode cryprq_connect(CrypRqHandle *handle, const CrypRqPeerParams *params);
CrypRqErrorCode cryprq_read_packet(CrypRqHandle *handle, uint8_t *buffer, size_t len, size_t *out_len);
CrypRqErrorCode cryprq_write_packet(CrypRqHandle *handle, const uint8_t *buffer, size_t len);
CrypRqErrorCode cryprq_on_network_change(CrypRqHandle *handle);
CrypRqErrorCode cryprq_close(CrypRqHandle *handle);
```

- `cryprq_init`: creates a runtime handle. Pass optional log level (e.g. `"info"`) and an allowlist of peer IDs for ACL enforcement. Returns ownership of an opaque pointer that must be closed with `cryprq_close`.
- `cryprq_connect`: starts a listener (`mode = LISTEN`) or dials a peer (`mode = DIAL`) using a libp2p multiaddr (e.g. `/ip4/0.0.0.0/udp/9999/quic-v1`).
- `cryprq_read_packet` / `cryprq_write_packet`: currently return `CRYPRQ_ERR_UNSUPPORTED` until the data-plane ships.
- `cryprq_on_network_change`: placeholder hook (returns `CRYPRQ_OK`) for the host to call when network conditions change.
- `cryprq_close`: releases resources and stops any active tasks.

### Memory & Ownership

- All strings passed into the API must be UTF-8 encoded and remain valid for the duration of the call.
- `CrypRqConfig.allow_peers` borrows memory; the caller owns the backing storage.
- `cryprq_init` allocates a handle (`Box<CrypRqHandle>`). The host must call `cryprq_close` exactly once.
- The library never takes ownership of buffers passed to `read_packet` / `write_packet`.

### Threading

- The FFI is thread-safe. Multiple threads may call the exported functions, but only one connection (`listen` or `dial`) can be active per handle.
- Logging is initialised on first `cryprq_init` and uses `env_logger`. Set `log_level` or the `RUST_LOG` environment variable before calling `cryprq_init` for deterministic output.

## Error Handling

Every function returns a `CrypRqErrorCode`. A value of `CRYPRQ_OK` indicates success; other codes denote recoverable issues (null pointers, invalid UTF-8, unsupported operations) or internal failures. Hosts should map the codes to their platform-specific error reporting.

## Deterministic Builds

- `cryp-rq-core` is part of the main Cargo workspace and inherits reproducible build tooling.
- Use `cargo build --release -p cryp-rq-core --target <triple>` to produce static libraries.
- CI (`.github/workflows/ci.yml`) cross-compiles the crate with `cargo check` for Apple, Android, and Windows targets to detect regressions.
- For distribution, combine the artefacts from `finish_qa_and_package.sh` with the generated headers/notarised binaries per platform.

