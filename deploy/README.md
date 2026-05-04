# Deploy

Production deployment artifacts for Sentrix Explorer V2.

```
deploy/
├── Caddyfile                       reverse-proxy block (host-side)
├── sentrix-explorer@.service       systemd template — one instance per network
└── deploy.sh                       build-and-ship from the build machine
```

## One-time host setup

```sh
# Caddy
sudo apt install caddy
# merge deploy/Caddyfile into /etc/caddy/Caddyfile
sudo systemctl reload caddy

# Service user + paths
sudo adduser --system --no-create-home --group www-data || true
sudo mkdir -p /var/www/sentrix-explorer/{mainnet,testnet}
sudo chown -R www-data:www-data /var/www/sentrix-explorer

# Per-instance port override (optional — defaults to :3000 mainnet, :3001 testnet)
sudo mkdir -p /etc/sentrix-explorer
sudo tee /etc/sentrix-explorer/testnet.env <<'EOF'
LEPTOS_SITE_ADDR=127.0.0.1:3001
EOF

# Systemd template
sudo cp deploy/sentrix-explorer@.service /etc/systemd/system/
sudo systemctl daemon-reload
```

## Cloudflare

- DNS: A `scan` and A `scan-testnet` → host IP, **proxied** (orange cloud)
- SSL/TLS mode: **Full** or **Full (Strict)**
- HTTP/3: enabled in network settings (CF auto)

## Deploy

```sh
# from the build machine (with cargo-leptos installed)
./deploy/deploy.sh
```

The script builds once per network (compile-time `SENTRIX_NETWORK` baked
in), rsyncs both bundles into separate per-instance directories on the
host, and restarts each systemd instance.
