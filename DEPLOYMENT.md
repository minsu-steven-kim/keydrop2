# Keydrop Deployment Guide

This guide covers deploying the Keydrop sync backend and Android app.

## Table of Contents

1. [Backend Deployment](#backend-deployment)
2. [Android App Deployment](#android-app-deployment)
3. [Infrastructure Setup](#infrastructure-setup)
4. [Security Checklist](#security-checklist)

---

## Backend Deployment

### Prerequisites

- Rust 1.75+ (for building)
- PostgreSQL 14+
- S3-compatible storage (AWS S3, MinIO, Cloudflare R2, etc.)
- A server/container runtime (Docker, Kubernetes, or bare metal)

### Option 1: Docker Deployment (Recommended)

#### 1. Create Production Dockerfile

```dockerfile
# backend/Dockerfile
FROM rust:1.75-slim-bookworm AS builder

WORKDIR /app
COPY . .

# Build release binary
RUN cargo build --release --package keydrop-backend

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/keydrop-backend /usr/local/bin/

# Create non-root user
RUN useradd -r -s /bin/false keydrop
USER keydrop

EXPOSE 3000

CMD ["keydrop-backend"]
```

#### 2. Create Production Docker Compose

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  backend:
    build:
      context: .
      dockerfile: Dockerfile
    environment:
      DATABASE_URL: postgres://keydrop:${DB_PASSWORD}@postgres:5432/keydrop
      JWT_SECRET: ${JWT_SECRET}
      AWS_ACCESS_KEY_ID: ${AWS_ACCESS_KEY_ID}
      AWS_SECRET_ACCESS_KEY: ${AWS_SECRET_ACCESS_KEY}
      AWS_REGION: ${AWS_REGION}
      S3_BUCKET: ${S3_BUCKET}
      RUST_LOG: keydrop_backend=info,tower_http=info
    ports:
      - "3000:3000"
    depends_on:
      postgres:
        condition: service_healthy
    restart: unless-stopped
    deploy:
      resources:
        limits:
          memory: 512M

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: keydrop
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_DB: keydrop
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U keydrop"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped
    deploy:
      resources:
        limits:
          memory: 256M

volumes:
  postgres_data:
```

#### 3. Deploy

```bash
# Create .env file with production secrets
cat > .env.prod << EOF
DB_PASSWORD=$(openssl rand -base64 32)
JWT_SECRET=$(openssl rand -base64 64)
AWS_ACCESS_KEY_ID=your-access-key
AWS_SECRET_ACCESS_KEY=your-secret-key
AWS_REGION=us-east-1
S3_BUCKET=keydrop-vault-blobs
EOF

# Deploy
docker compose -f docker-compose.prod.yml --env-file .env.prod up -d
```

### Option 2: Kubernetes Deployment

#### 1. Create Kubernetes Manifests

```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: keydrop
---
# k8s/secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: keydrop-secrets
  namespace: keydrop
type: Opaque
stringData:
  database-url: postgres://keydrop:PASSWORD@postgres:5432/keydrop
  jwt-secret: your-jwt-secret-here
  aws-access-key-id: your-access-key
  aws-secret-access-key: your-secret-key
---
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: keydrop-backend
  namespace: keydrop
spec:
  replicas: 2
  selector:
    matchLabels:
      app: keydrop-backend
  template:
    metadata:
      labels:
        app: keydrop-backend
    spec:
      containers:
      - name: backend
        image: your-registry/keydrop-backend:latest
        ports:
        - containerPort: 3000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: keydrop-secrets
              key: database-url
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: keydrop-secrets
              key: jwt-secret
        - name: AWS_ACCESS_KEY_ID
          valueFrom:
            secretKeyRef:
              name: keydrop-secrets
              key: aws-access-key-id
        - name: AWS_SECRET_ACCESS_KEY
          valueFrom:
            secretKeyRef:
              name: keydrop-secrets
              key: aws-secret-access-key
        - name: AWS_REGION
          value: "us-east-1"
        - name: S3_BUCKET
          value: "keydrop-vault-blobs"
        - name: RUST_LOG
          value: "keydrop_backend=info"
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /api/v1/health
            port: 3000
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /api/v1/health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 10
---
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: keydrop-backend
  namespace: keydrop
spec:
  selector:
    app: keydrop-backend
  ports:
  - port: 80
    targetPort: 3000
  type: ClusterIP
---
# k8s/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: keydrop-ingress
  namespace: keydrop
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - api.keydrop.app
    secretName: keydrop-tls
  rules:
  - host: api.keydrop.app
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: keydrop-backend
            port:
              number: 80
```

#### 2. Deploy to Kubernetes

```bash
kubectl apply -f k8s/
```

### Option 3: Cloud Platform Deployment

#### AWS (ECS/Fargate)

1. Push Docker image to ECR
2. Create ECS task definition
3. Create ECS service with ALB
4. Use RDS for PostgreSQL
5. Use S3 for blob storage

#### Google Cloud (Cloud Run)

```bash
# Build and push
gcloud builds submit --tag gcr.io/PROJECT_ID/keydrop-backend

# Deploy
gcloud run deploy keydrop-backend \
  --image gcr.io/PROJECT_ID/keydrop-backend \
  --platform managed \
  --region us-central1 \
  --set-env-vars "DATABASE_URL=..." \
  --set-secrets "JWT_SECRET=jwt-secret:latest"
```

#### Fly.io (Simple deployment)

```toml
# fly.toml
app = "keydrop-backend"
primary_region = "iad"

[build]
  dockerfile = "Dockerfile"

[env]
  RUST_LOG = "keydrop_backend=info"

[http_service]
  internal_port = 3000
  force_https = true

[[services.ports]]
  port = 443
  handlers = ["tls", "http"]
```

```bash
fly launch
fly secrets set JWT_SECRET=$(openssl rand -base64 64)
fly secrets set DATABASE_URL=postgres://...
fly deploy
```

### Health Check Endpoint

The backend includes a health check endpoint at `/api/v1/health` for load balancer and container orchestration health checks. This endpoint returns `"OK"` when the service is running.

---

## Android App Deployment

### Prerequisites

- Android Studio Arctic Fox+
- JDK 17
- Android SDK 34
- Signing keystore

### 1. Configure Signing

```kotlin
// app/build.gradle.kts
android {
    signingConfigs {
        create("release") {
            storeFile = file(System.getenv("KEYSTORE_FILE") ?: "keystore.jks")
            storePassword = System.getenv("KEYSTORE_PASSWORD")
            keyAlias = System.getenv("KEY_ALIAS")
            keyPassword = System.getenv("KEY_PASSWORD")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            signingConfig = signingConfigs.getByName("release")
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
}
```

### 2. Generate Signing Key

```bash
keytool -genkey -v -keystore keydrop-release.jks \
  -keyalg RSA -keysize 2048 -validity 10000 \
  -alias keydrop
```

### 3. Update API Base URL

```kotlin
// di/NetworkModule.kt
@Module
@InstallIn(SingletonComponent::class)
object NetworkModule {
    // Production URL
    private const val BASE_URL = "https://api.keydrop.app/"

    // ...
}
```

### 4. Build Release APK/AAB

```bash
# Set environment variables
export KEYSTORE_FILE=/path/to/keydrop-release.jks
export KEYSTORE_PASSWORD=your-store-password
export KEY_ALIAS=keydrop
export KEY_PASSWORD=your-key-password

# Build AAB for Play Store
./gradlew bundleRelease

# Build APK for direct distribution
./gradlew assembleRelease
```

### 5. Build Native Libraries (JNI)

Before release, build the native crypto libraries:

```bash
# Install Android NDK and targets
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

# Install cargo-ndk
cargo install cargo-ndk

# Build for all Android ABIs
cd crypto-core/uniffi
cargo ndk -t armeabi-v7a -t arm64-v8a -t x86 -t x86_64 -o ../android/app/src/main/jniLibs build --release

# Generate Kotlin bindings
cargo run --bin uniffi-bindgen generate src/crypto_core.udl --language kotlin --out-dir ../android/app/src/main/java/
```

### 6. Play Store Deployment

1. Create app in [Google Play Console](https://play.google.com/console)
2. Upload AAB to Internal Testing track
3. Complete store listing:
   - App name: Keydrop
   - Short description
   - Full description
   - Screenshots (phone, tablet)
   - Feature graphic
   - Privacy policy URL
4. Complete content rating questionnaire
5. Set up pricing and distribution
6. Submit for review

### 7. Alternative Distribution

#### GitHub Releases

```bash
# Create release with APK
gh release create v1.0.0 \
  app/build/outputs/apk/release/app-release.apk \
  --title "Keydrop v1.0.0" \
  --notes "Initial release"
```

#### F-Droid

Create `fdroid/metadata/com.keydrop.yml`:
```yaml
Categories:
  - Security
License: MIT
SourceCode: https://github.com/yourname/keydrop
IssueTracker: https://github.com/yourname/keydrop/issues

AutoName: Keydrop
Description: |
  Secure password manager with zero-knowledge sync.

  Features:
  * End-to-end encryption
  * Biometric unlock
  * Android Autofill
  * Cross-device sync

RepoType: git
Repo: https://github.com/yourname/keydrop.git

Builds:
  - versionName: 1.0.0
    versionCode: 1
    commit: v1.0.0
    subdir: android/app
    gradle:
      - yes
```

---

## Infrastructure Setup

### Database (PostgreSQL)

#### Managed Options (Recommended)
- **AWS RDS**: Automated backups, Multi-AZ
- **Google Cloud SQL**: Easy scaling
- **Supabase**: Free tier available
- **Neon**: Serverless PostgreSQL

#### Self-Hosted
```bash
# Create database
psql -U postgres -c "CREATE DATABASE keydrop;"
psql -U postgres -c "CREATE USER keydrop WITH PASSWORD 'secure-password';"
psql -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE keydrop TO keydrop;"

# Enable SSL
# Edit postgresql.conf:
ssl = on
ssl_cert_file = '/path/to/server.crt'
ssl_key_file = '/path/to/server.key'
```

### Blob Storage (S3-Compatible)

#### AWS S3
```bash
# Create bucket
aws s3 mb s3://keydrop-vault-blobs --region us-east-1

# Set bucket policy (private)
aws s3api put-public-access-block \
  --bucket keydrop-vault-blobs \
  --public-access-block-configuration \
  "BlockPublicAcls=true,IgnorePublicAcls=true,BlockPublicPolicy=true,RestrictPublicBuckets=true"

# Enable versioning (optional, for recovery)
aws s3api put-bucket-versioning \
  --bucket keydrop-vault-blobs \
  --versioning-configuration Status=Enabled

# Enable encryption
aws s3api put-bucket-encryption \
  --bucket keydrop-vault-blobs \
  --server-side-encryption-configuration \
  '{"Rules":[{"ApplyServerSideEncryptionByDefault":{"SSEAlgorithm":"AES256"}}]}'
```

#### Cloudflare R2 (S3-compatible, no egress fees)
```bash
# Set environment variables
S3_ENDPOINT=https://ACCOUNT_ID.r2.cloudflarestorage.com
AWS_ACCESS_KEY_ID=your-r2-access-key
AWS_SECRET_ACCESS_KEY=your-r2-secret-key
S3_BUCKET=keydrop-vault-blobs
```

#### MinIO (Self-hosted)
```yaml
# docker-compose.yml addition
minio:
  image: minio/minio:latest
  command: server /data --console-address ":9001"
  environment:
    MINIO_ROOT_USER: ${MINIO_ROOT_USER}
    MINIO_ROOT_PASSWORD: ${MINIO_ROOT_PASSWORD}
  volumes:
    - minio_data:/data
  ports:
    - "9000:9000"
    - "9001:9001"
```

### Reverse Proxy (Nginx)

```nginx
# /etc/nginx/sites-available/keydrop
server {
    listen 443 ssl http2;
    server_name api.keydrop.app;

    ssl_certificate /etc/letsencrypt/live/api.keydrop.app/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.keydrop.app/privkey.pem;

    # Modern TLS configuration
    ssl_protocols TLSv1.3;
    ssl_prefer_server_ciphers off;

    # Security headers
    add_header Strict-Transport-Security "max-age=63072000" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "DENY" always;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket timeout
        proxy_read_timeout 86400;
    }
}

server {
    listen 80;
    server_name api.keydrop.app;
    return 301 https://$server_name$request_uri;
}
```

### SSL/TLS Certificates

```bash
# Using Certbot
sudo certbot --nginx -d api.keydrop.app

# Auto-renewal
sudo certbot renew --dry-run
```

---

## Security Checklist

### Backend Security

- [ ] **JWT Secret**: Use a cryptographically random secret (64+ bytes)
  ```bash
  openssl rand -base64 64
  ```

- [ ] **Database**:
  - [ ] Strong password
  - [ ] SSL connections required
  - [ ] Network isolation (VPC)
  - [ ] Regular backups

- [ ] **S3 Bucket**:
  - [ ] Private access only
  - [ ] Server-side encryption enabled
  - [ ] Versioning enabled (for recovery)
  - [ ] Access logging enabled

- [ ] **Network**:
  - [ ] TLS 1.3 only
  - [ ] HSTS enabled
  - [ ] Rate limiting configured
  - [ ] DDoS protection (Cloudflare, AWS Shield)

- [ ] **Monitoring**:
  - [ ] Error tracking (Sentry)
  - [ ] Request logging
  - [ ] Alerting on anomalies

### Android App Security

- [ ] **Code**:
  - [ ] ProGuard/R8 obfuscation enabled
  - [ ] No hardcoded secrets
  - [ ] Certificate pinning implemented

- [ ] **Storage**:
  - [ ] EncryptedSharedPreferences for sensitive data
  - [ ] Room database encryption (SQLCipher)
  - [ ] No backup allowed (android:allowBackup="false")

- [ ] **Runtime**:
  - [ ] FLAG_SECURE on sensitive screens
  - [ ] Root/Jailbreak detection
  - [ ] Debugger detection

### Pre-Launch Checklist

1. [ ] Run security scan (OWASP ZAP, Burp Suite)
2. [ ] Penetration testing
3. [ ] Load testing (k6, locust)
4. [ ] Database backup/restore test
5. [ ] Disaster recovery drill
6. [ ] Privacy policy published
7. [ ] Terms of service published

---

## Environment Variables Reference

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://user:pass@host:5432/db` |
| `JWT_SECRET` | Secret for signing JWTs | (64+ random bytes, base64) |
| `AWS_ACCESS_KEY_ID` | S3 access key | `AKIA...` |
| `AWS_SECRET_ACCESS_KEY` | S3 secret key | (secret) |
| `AWS_REGION` | S3 region | `us-east-1` |
| `S3_BUCKET` | S3 bucket name | `keydrop-vault-blobs` |
| `S3_ENDPOINT` | Custom S3 endpoint (MinIO/R2) | `http://minio:9000` |
| `RUST_LOG` | Log level | `keydrop_backend=info` |

---

## Monitoring & Operations

### Logging

Configure structured logging for production:

```rust
// In main.rs
tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer().json())
    .with(tracing_subscriber::EnvFilter::from_default_env())
    .init();
```

### Metrics

Add Prometheus metrics endpoint:

```rust
// Add to Cargo.toml
// metrics = "0.21"
// metrics-exporter-prometheus = "0.12"

// In main.rs
let recorder = PrometheusBuilder::new().build_recorder();
let handle = recorder.handle();
metrics::set_boxed_recorder(Box::new(recorder)).unwrap();

// Add /metrics endpoint
.route("/metrics", get(|| async move { handle.render() }))
```

### Backup Strategy

```bash
# Daily PostgreSQL backup
pg_dump -Fc keydrop > backup_$(date +%Y%m%d).dump

# Restore
pg_restore -d keydrop backup_20240101.dump

# S3 versioning handles blob backups automatically
```

---

## Cost Estimation

### Small Scale (< 1000 users)
- **Fly.io**: ~$5-10/month
- **Supabase (PostgreSQL)**: Free tier
- **Cloudflare R2**: Free tier (10GB)
- **Total**: ~$5-10/month

### Medium Scale (1000-10000 users)
- **AWS ECS Fargate**: ~$30/month
- **AWS RDS (db.t3.micro)**: ~$15/month
- **AWS S3**: ~$5/month
- **Total**: ~$50/month

### Large Scale (10000+ users)
- **Kubernetes cluster**: ~$100-300/month
- **Managed PostgreSQL**: ~$50-100/month
- **S3/R2**: ~$20-50/month
- **Total**: ~$200-500/month
