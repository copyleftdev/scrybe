# Scrybe Docker Sandbox

Complete Docker Compose environment for testing Scrybe locally.

## Services

| Service | Port | Description |
|---------|------|-------------|
| Redis | 6379 | Session cache & rate limiting |
| ClickHouse | 8123, 9000 | Analytical database |
| Gateway | 8080 | Scrybe ingestion API |
| Test App | 3000 | Demo web application |
| Dashboard | 8088 | Simple monitoring UI |

## Quick Start

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop all services
docker-compose down

# Stop and remove volumes
docker-compose down -v
```

## Access Points

- **Test App**: http://localhost:3000
- **API Gateway**: http://localhost:8080
- **Dashboard**: http://localhost:8088
- **ClickHouse HTTP**: http://localhost:8123
- **Redis**: localhost:6379

## Testing the Stack

1. **Open Test App**: http://localhost:3000
2. **Accept consent** in the banner
3. **Interact with the page** (move mouse, scroll, click)
4. **Check Dashboard**: http://localhost:8088
5. **Query ClickHouse directly**:
```bash
curl "http://localhost:8123/?query=SELECT count() FROM scrybe.sessions FORMAT JSON"
```

## Health Checks

```bash
# Gateway health
curl http://localhost:8080/health

# ClickHouse health  
curl http://localhost:8123/ping

# Redis health
redis-cli -h localhost ping
```

## Environment Variables

Edit `docker-compose.yml` to customize:

- `SCRYBE_HMAC_KEY`: Authentication key (change for production!)
- `SCRYBE_LOG_LEVEL`: `debug`, `info`, `warn`, `error`
- `CLICKHOUSE_PASSWORD`: Database password

## Development

```bash
# Rebuild gateway after code changes
docker-compose build gateway
docker-compose up -d gateway

# View gateway logs
docker-compose logs -f gateway

# Execute ClickHouse queries
docker-compose exec clickhouse clickhouse-client -d scrybe
```

## Troubleshooting

**Gateway won't start:**
```bash
# Check dependencies
docker-compose ps
docker-compose logs redis
docker-compose logs clickhouse
```

**No data in ClickHouse:**
```bash
# Check if schema initialized
docker-compose exec clickhouse clickhouse-client -q "SHOW TABLES FROM scrybe"

# Manually run init script
docker-compose exec clickhouse clickhouse-client < deployment/clickhouse/init.sql
```

**Test app can't reach gateway:**
```bash
# Check network
docker network inspect scrybe_scrybe-network

# Check gateway logs
docker-compose logs gateway
```

## Production Notes

⚠️ **This is a development environment!**

For production:
1. Change all passwords and keys
2. Enable TLS/SSL
3. Configure proper authentication
4. Set up monitoring and alerts
5. Use production-grade database configs
6. Enable Redis persistence
7. Configure ClickHouse replication

## Architecture

```
┌─────────────┐
│  Test App   │ :3000
│  (Browser)  │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Gateway   │ :8080
│  (Rust API) │
└──┬───────┬──┘
   │       │
   ▼       ▼
┌──────┐ ┌───────────┐
│Redis │ │ ClickHouse│
│:6379 │ │ :8123     │
└──────┘ └───────────┘
```

## Cleaning Up

```bash
# Stop and remove everything
docker-compose down -v --rmi all

# Remove orphaned volumes
docker volume prune
```

## Next Steps

- Customize test app in `deployment/test-app/`
- Add custom queries to dashboard
- Configure alerts and monitoring
- Scale services with `docker-compose scale`
