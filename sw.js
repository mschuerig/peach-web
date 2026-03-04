// sw.js — Service Worker for Peach ear training app
const CACHE_VERSION = 'peach-v1';

self.addEventListener('install', (event) => {
  // Pre-cache only the HTML shell (stable paths)
  event.waitUntil(
    caches.open(CACHE_VERSION)
      .then(cache => cache.addAll(['/', '/index.html']))
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

  // SPA navigation: return cached index.html for all navigation requests.
  // Routes like /profile, /settings, /training/comparison are handled by
  // the Leptos client-side router, not by actual files on the server.
  if (event.request.mode === 'navigate') {
    event.respondWith(
      caches.match('/index.html').then(cached => cached || fetch(event.request))
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
          if (!response || response.status !== 200 || response.type !== 'basic') {
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
