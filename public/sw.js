// Sentrix Explorer service worker.
//
// Cache strategy:
//   - /pkg/*           → cache-first (immutable bundle, content-hashed)
//   - /icon*, /manifest → cache-first (rarely changes)
//   - / and /assets    → stale-while-revalidate (HTML may update on deploy)
//   - everything else  → network-first (gRPC-Web, /api, etc.)
//
// Bumping CACHE_VERSION invalidates all old caches on next activate.

const CACHE_VERSION = "v2";
const STATIC_CACHE = `sentrix-explorer-static-${CACHE_VERSION}`;
const RUNTIME_CACHE = `sentrix-explorer-runtime-${CACHE_VERSION}`;

const PRECACHE = ["/", "/manifest.json", "/icon.svg", "/icon-maskable.svg"];

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(STATIC_CACHE).then((cache) => cache.addAll(PRECACHE)),
  );
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    (async () => {
      const keys = await caches.keys();
      await Promise.all(
        keys
          .filter((k) => k !== STATIC_CACHE && k !== RUNTIME_CACHE)
          .map((k) => caches.delete(k)),
      );
      await self.clients.claim();
    })(),
  );
});

self.addEventListener("fetch", (event) => {
  const req = event.request;
  if (req.method !== "GET") return;

  const url = new URL(req.url);

  // Never intercept gRPC-Web — those POSTs need to pass straight through,
  // and even GETs to chain endpoints should bypass the cache.
  if (url.host !== self.location.host) return;
  if (url.pathname.startsWith("/api/") || url.pathname.startsWith("/rpc")) return;

  // Cache-first for the WASM bundle and static icons.
  if (url.pathname.startsWith("/pkg/") || url.pathname.startsWith("/icon")) {
    event.respondWith(cacheFirst(req));
    return;
  }

  // Stale-while-revalidate for navigation HTML.
  if (req.mode === "navigate") {
    event.respondWith(staleWhileRevalidate(req));
    return;
  }
});

async function cacheFirst(req) {
  const cache = await caches.open(STATIC_CACHE);
  const hit = await cache.match(req);
  if (hit) return hit;
  const res = await fetch(req);
  if (res.ok) cache.put(req, res.clone());
  return res;
}

async function staleWhileRevalidate(req) {
  const cache = await caches.open(RUNTIME_CACHE);
  const hit = await cache.match(req);
  const fetched = fetch(req)
    .then((res) => {
      if (res.ok) cache.put(req, res.clone());
      return res;
    })
    .catch(() => hit);
  return hit || fetched;
}
