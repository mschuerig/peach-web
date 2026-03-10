// sw.js — Service Worker for Peach ear training app
const CACHE_VERSION = 'peach-v1'; // Replaced at build time by CI (stays peach-v1 in local dev)

self.addEventListener('install', (event) => {
  // Pre-cache only the HTML shell (stable paths)
  event.waitUntil(
    caches.open(CACHE_VERSION)
      .then(cache => cache.addAll(['./index.html']))
      .then(() => self.skipWaiting())
  );
});

self.addEventListener('activate', (event) => {
  // Delete old caches and take control of all clients immediately
  event.waitUntil(
    caches.keys()
      .then(keys => Promise.all(
        keys.filter(k => k !== CACHE_VERSION).map(k => caches.delete(k))
      ))
      .then(() => self.clients.claim())
  );
});

self.addEventListener('fetch', (event) => {
  // Only handle GET requests
  if (event.request.method !== 'GET') return;

  // SPA navigation: network-first for index.html so online users always get
  // the latest version. Offline fallback serves the cached copy.
  if (event.request.mode === 'navigate') {
    event.respondWith(
      fetch(event.request)
        .then(response => {
          if (response.ok) {
            const clone = response.clone();
            caches.open(CACHE_VERSION)
              .then(cache => cache.put('./index.html', clone))
              .catch(() => {});
          }
          return response;
        })
        .catch(() => caches.match('./index.html'))
    );
    return;
  }

  // All other requests (hashed assets, SoundFont, worklet files):
  // cache-first with network fallback, caching responses on first fetch.
  event.respondWith(
    caches.match(event.request)
      .then(cached => {
        if (cached) return cached;
        return fetch(event.request).then(response => {
          // Don't cache non-ok responses or opaque responses
          if (!response || response.status !== 200 || response.type === 'opaque') {
            return response;
          }
          const clone = response.clone();
          caches.open(CACHE_VERSION)
            .then(cache => cache.put(event.request, clone))
            .catch(() => {}); // Gracefully handle cache storage quota errors
          return response;
        });
      })
  );
});
