# Scrybe Gateway

HTTP ingestion gateway for receiving browser session data using Axum.

## Features

- **Axum HTTP server** with async request handling
- **Health check endpoints** for Kubernetes liveness/readiness probes
- **Graceful shutdown** with signal handling (SIGTERM/SIGINT)
- **TigerStyle-compliant** error handling

## Endpoints

### Health Checks

- `GET /health` - Liveness probe (always returns 200 OK if running)
- `GET /health/ready` - Readiness probe (checks dependencies)

## Running

```bash
# Set environment variables
export SCRYBE_HOST=127.0.0.1
export SCRYBE_PORT=8080

# Run the gateway
cargo run -p scrybe-gateway
```

## Configuration

All configuration is loaded from environment variables:

- `SCRYBE_HOST` - Server host (default: 127.0.0.1)
- `SCRYBE_PORT` - Server port (default: 8080)
- `SCRYBE_MAX_CONNECTIONS` - Max concurrent connections (default: 10000)
- `SCRYBE_ENABLE_TLS` - Enable TLS (default: true)
- `SCRYBE_REQUEST_TIMEOUT_SECS` - Request timeout (default: 30)

## Graceful Shutdown

The gateway handles SIGTERM and SIGINT signals gracefully:

1. Stops accepting new connections
2. Waits for in-flight requests to complete (30s timeout)
3. Closes all resources cleanly

## TigerStyle Compliance

- ✅ No `unwrap()` or `panic!()` in production code
- ✅ Explicit error handling
- ✅ All functions documented
