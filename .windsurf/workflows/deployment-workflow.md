# Production Deployment Workflow

**Description**: Safe, repeatable deployment process for Scrybe services to production

## Pre-Deployment Checklist

Before starting deployment:
- [ ] All tests pass in CI/CD
- [ ] Security audit completed and cleared
- [ ] Performance benchmarks meet targets
- [ ] Code review approved
- [ ] CHANGELOG.md updated
- [ ] Release notes prepared
- [ ] Rollback plan documented
- [ ] On-call engineer identified
- [ ] Deployment window scheduled

## Workflow Steps

### 1. Version Bump & Tagging

```bash
# Ensure on main branch and up to date
git checkout main
git pull origin main

# Determine version (semver: MAJOR.MINOR.PATCH)
# Breaking changes: MAJOR
# New features: MINOR
# Bug fixes: PATCH

# Update version in Cargo.toml
VERSION="0.3.0"
sed -i "s/^version = .*/version = \"$VERSION\"/" Cargo.toml

# Update package.json for SDK
cd sdk/
npm version $VERSION --no-git-tag-version
cd ..

# Commit version bump
git add Cargo.toml sdk/package.json CHANGELOG.md
git commit -m "chore: bump version to v$VERSION"

# Create git tag
git tag -a "v$VERSION" -m "Release v$VERSION"

# Push tag to trigger release
git push origin main --tags
```

### 2. Build Release Artifacts

#### A. Rust Binary
```bash
# Build optimized release binary
cargo build --release --workspace

# Strip debug symbols to reduce size
strip target/release/scrybe-server

# Verify binary
./target/release/scrybe-server --version

# Run smoke test
./target/release/scrybe-server --check-config config/production.toml
```

#### B. Docker Image
```bash
# Build production Docker image
docker build -t scrybe/server:$VERSION -f Dockerfile.prod .

# Tag as latest
docker tag scrybe/server:$VERSION scrybe/server:latest

# Scan for vulnerabilities
docker scan scrybe/server:$VERSION

# Push to registry
docker push scrybe/server:$VERSION
docker push scrybe/server:latest
```

#### C. JavaScript SDK
```bash
cd sdk/

# Install dependencies
npm ci

# Build production bundle
npm run build

# Verify bundle
ls -lh dist/scrybe-sdk.min.js

# Check bundle size (<50KB gzipped)
gzip -c dist/scrybe-sdk.min.js | wc -c

# Publish to npm (if applicable)
npm publish --access public

cd ..
```

### 3. Database Migration (If Needed)

#### A. ClickHouse Schema Changes
```bash
# Connect to production ClickHouse
clickhouse-client --host prod-clickhouse.scrybe.io --secure

# Run migration in transaction
BEGIN TRANSACTION;

-- Apply schema changes
ALTER TABLE sessions ADD COLUMN new_field String;

-- Verify changes
DESCRIBE TABLE sessions;

-- If successful, commit
COMMIT;

-- If issues, rollback
-- ROLLBACK;
```

#### B. Backup Before Migration
```bash
# Create backup before schema changes
clickhouse-client --query "
  CREATE TABLE sessions_backup AS sessions;
  INSERT INTO sessions_backup SELECT * FROM sessions;
"

# Document backup location
echo "Backup created: sessions_backup at $(date)" >> deployment-log.txt
```

### 4. Infrastructure Preparation

#### A. Update Configuration
```bash
# Update production config
cat > config/production.toml << 'EOF'
[server]
host = "0.0.0.0"
port = 8080
workers = 8

[database]
clickhouse_url = "https://clickhouse.prod.scrybe.io:8443"
redis_url = "redis://redis.prod.scrybe.io:6379"

[security]
hmac_key = "${SCRYBE_HMAC_KEY}"
tls_cert = "/etc/scrybe/certs/server.crt"
tls_key = "/etc/scrybe/certs/server.key"

[limits]
max_connections = 10000
rate_limit_per_ip = 100
EOF

# Validate configuration
./target/release/scrybe-server --check-config config/production.toml
```

#### B. Update Kubernetes Manifests
```bash
# Update deployment manifest
cat > k8s/deployment.yaml << EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: scrybe-server
  namespace: production
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: scrybe-server
  template:
    metadata:
      labels:
        app: scrybe-server
        version: $VERSION
    spec:
      containers:
      - name: scrybe-server
        image: scrybe/server:$VERSION
        ports:
        - containerPort: 8080
        env:
        - name: SCRYBE_HMAC_KEY
          valueFrom:
            secretKeyRef:
              name: scrybe-secrets
              key: hmac-key
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
EOF
```

### 5. Blue-Green Deployment

#### A. Deploy to Green Environment
```bash
# Deploy new version to green environment
kubectl apply -f k8s/deployment-green.yaml

# Wait for deployment to be ready
kubectl rollout status deployment/scrybe-server-green -n production

# Verify pods are running
kubectl get pods -n production -l app=scrybe-server,env=green
```

#### B. Smoke Test Green Environment
```bash
# Get green service endpoint
GREEN_ENDPOINT=$(kubectl get svc scrybe-server-green -n production -o jsonpath='{.status.loadBalancer.ingress[0].hostname}')

# Run smoke tests
curl -f https://$GREEN_ENDPOINT/health || exit 1
curl -f https://$GREEN_ENDPOINT/metrics || exit 1

# Run synthetic transactions
./scripts/smoke-test.sh $GREEN_ENDPOINT
```

#### C. Switch Traffic to Green
```bash
# Update load balancer to point to green
kubectl patch svc scrybe-server-lb -n production -p '
{
  "spec": {
    "selector": {
      "app": "scrybe-server",
      "env": "green"
    }
  }
}'

# Monitor for 5 minutes
watch -n 10 'kubectl get pods -n production -l app=scrybe-server'
```

### 6. Canary Deployment (Alternative Strategy)

#### A. Deploy Canary (10% Traffic)
```bash
# Update canary deployment
kubectl apply -f k8s/canary-deployment.yaml

# Configure traffic split (90% stable, 10% canary)
kubectl apply -f k8s/virtual-service.yaml
```

#### B. Monitor Canary Metrics
```bash
# Watch error rates
kubectl logs -f -l app=scrybe-server,version=canary -n production

# Check metrics in Prometheus
# Error rate should be < 0.1%
# Latency p99 should be < 10ms

# Monitor for 30 minutes
```

#### C. Promote or Rollback
```bash
# If metrics good, promote to 100%
kubectl apply -f k8s/virtual-service-full.yaml

# Or rollback if issues
kubectl apply -f k8s/virtual-service-rollback.yaml
```

### 7. Post-Deployment Verification

#### A. Health Checks
```bash
# Verify all endpoints
PROD_URL="https://api.scrybe.io"

curl -f $PROD_URL/health
curl -f $PROD_URL/metrics
curl -f $PROD_URL/ready

# Check version
curl $PROD_URL/version | jq '.version'
```

#### B. Integration Tests
```bash
# Run production integration tests (read-only)
./scripts/prod-integration-test.sh

# Verify data flow
# 1. Send test telemetry
# 2. Query for stored data
# 3. Verify fingerprint generation
```

#### C. Monitor Key Metrics
```bash
# Check dashboard for:
# - Request rate (should be normal)
# - Error rate (should be < 0.1%)
# - Latency p50, p95, p99 (should be < 10ms)
# - CPU/Memory usage (should be < 80%)

# Watch logs for errors
kubectl logs -f -l app=scrybe-server -n production --tail=100
```

### 8. Database Health Check

```bash
# Check ClickHouse
clickhouse-client --host prod-clickhouse.scrybe.io --query "
  SELECT 
    count() as total_sessions,
    uniq(fingerprint_hash) as unique_fingerprints,
    max(created_at) as latest_session
  FROM sessions
  WHERE created_at > now() - INTERVAL 1 HOUR;
"

# Check Redis
redis-cli -h redis.prod.scrybe.io INFO stats
redis-cli -h redis.prod.scrybe.io DBSIZE
```

### 9. Gradual Rollout (For Major Changes)

```bash
# Phase 1: Deploy to 10% of users
# Monitor for 2 hours

# Phase 2: Increase to 25%
# Monitor for 2 hours

# Phase 3: Increase to 50%
# Monitor for 4 hours

# Phase 4: Increase to 100%
# Monitor for 24 hours
```

### 10. Documentation Updates

#### A. Update Deployment Log
```bash
cat >> deployment-log.txt << EOF
---
Deployment: v$VERSION
Date: $(date)
Deployed by: $(whoami)
Strategy: Blue-Green
Incidents: None
Rollback: Not required
Notes: Smooth deployment, all metrics normal
---
EOF
```

#### B. Update Status Page
```bash
# Post update to status page
curl -X POST https://status.scrybe.io/api/incidents \
  -H "Authorization: Bearer $STATUS_API_KEY" \
  -d '{
    "name": "Scheduled Deployment - v'$VERSION'",
    "status": "completed",
    "message": "Deployment completed successfully. All systems operational."
  }'
```

### 11. Communication

#### A. Notify Team
```bash
# Slack notification
curl -X POST $SLACK_WEBHOOK_URL \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "✅ Scrybe v'$VERSION' deployed to production",
    "blocks": [
      {
        "type": "section",
        "text": {
          "type": "mrkdwn",
          "text": "*Deployment Complete*\n• Version: v'$VERSION'\n• Status: Successful\n• Health: All systems operational"
        }
      }
    ]
  }'
```

#### B. Update Release Notes
- Post to GitHub Releases
- Update documentation site
- Send email to customers (if applicable)
- Tweet/announce (if major release)

### 12. Monitoring Period

Monitor for 24 hours after deployment:

#### Hour 1-2: Critical Watch
- Watch all metrics every 5 minutes
- Check logs for errors
- Verify functionality end-to-end

#### Hour 3-12: Active Monitoring
- Check metrics every 30 minutes
- Review error dashboards
- Respond to any alerts

#### Hour 13-24: Normal Monitoring
- Regular on-call monitoring
- Review daily metrics summary

### 13. Rollback Procedure (If Needed)

```bash
# Quick rollback to previous version
PREVIOUS_VERSION="0.2.0"

# Option 1: Kubernetes rollback
kubectl rollout undo deployment/scrybe-server -n production

# Option 2: Switch back to blue environment
kubectl patch svc scrybe-server-lb -n production -p '
{
  "spec": {
    "selector": {
      "app": "scrybe-server",
      "env": "blue"
    }
  }
}'

# Option 3: Deploy previous version
kubectl set image deployment/scrybe-server \
  scrybe-server=scrybe/server:$PREVIOUS_VERSION \
  -n production

# Verify rollback
kubectl rollout status deployment/scrybe-server -n production

# Notify team
echo "⚠️ Rolled back to v$PREVIOUS_VERSION due to [REASON]"
```

### 14. Post-Deployment Tasks

Day after deployment:
- [ ] Review incident reports
- [ ] Analyze performance metrics
- [ ] Check error logs summary
- [ ] Update runbooks if needed
- [ ] Document lessons learned
- [ ] Schedule retrospective (if issues)

## Deployment Checklist Summary

### Pre-Deployment
- [ ] All CI/CD checks pass
- [ ] Security audit complete
- [ ] Performance validated
- [ ] Rollback plan ready
- [ ] Team notified

### During Deployment
- [ ] Version tagged
- [ ] Artifacts built
- [ ] Database migrated (if needed)
- [ ] Deployed to staging first
- [ ] Smoke tests pass
- [ ] Gradual rollout
- [ ] Metrics monitored

### Post-Deployment
- [ ] Health checks pass
- [ ] Integration tests pass
- [ ] Metrics normal for 24h
- [ ] Documentation updated
- [ ] Team notified
- [ ] Release announced

## Emergency Hotfix Procedure

For critical production issues:

```bash
# Create hotfix branch
git checkout -b hotfix/v0.2.1 v0.2.0

# Make minimal fix
# Test thoroughly
# Update version to 0.2.1

# Fast-track deployment
# Skip canary, go straight to rollout
# Monitor closely

# After 1 hour of stability, merge back
git checkout main
git merge hotfix/v0.2.1
```

## Success Criteria

Deployment is successful when:
- ✅ All health checks pass
- ✅ Error rate < 0.1%
- ✅ Latency p99 < 10ms
- ✅ No alerts triggered
- ✅ Metrics stable for 24 hours
- ✅ No rollback required

Remember: Slow is smooth, smooth is fast. Take time to verify each step.
