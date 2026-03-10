---
stepsCompleted: [1, 2, 3, 4, 5, 6]
inputDocuments: []
workflowType: 'research'
lastStep: 6
research_type: 'technical'
research_topic: 'GitHub Pages caching and cache busting for Trunk-built Leptos WASM app'
research_goals: 'Verify current setup ensures assets are cached as long as possible but updated when a new version is deployed'
user_name: 'Michael'
date: '2026-03-10'
web_research_enabled: true
source_verification: true
---

# Research Report: GitHub Pages Caching for Trunk-built Leptos WASM App

**Date:** 2026-03-10
**Author:** Michael
**Research Type:** Technical

---

## Executive Summary

The peach-web deployment on GitHub Pages has a **critical service worker bug** that prevents users from receiving app updates. While Trunk's filename hashing and GitHub Pages' 10-minute cache TTL work correctly, the service worker's cache-first strategy for `index.html` combined with a static `CACHE_VERSION` means users are stuck on the version they first cached — indefinitely.

**Two targeted changes fix this completely:** (1) switch the service worker's navigation handler from cache-first to network-first, and (2) inject a build-derived hash into `CACHE_VERSION` via a CI pipeline step. All existing users will automatically migrate within ~10 minutes of their next visit after deployment. No other changes are needed — the rest of the caching infrastructure is correct.

## Research Overview

This report analyzes whether the current peach-web deployment setup on GitHub Pages ensures that users always receive the latest version of the app when online, while maximizing caching for performance. The analysis covers Trunk's cache busting, GitHub Pages caching behavior, and the service worker update lifecycle. See the Executive Summary above for key findings.

---

## Technical Research Scope Confirmation

**Research Topic:** GitHub Pages caching and cache busting for Trunk-built Leptos WASM app
**Research Goals:** Verify current setup ensures assets are cached as long as possible but updated when a new version is deployed

**Research Methodology:**

- Current web data with rigorous source verification
- Audit of project configuration files (Trunk.toml, ci.yml, sw.js, index.html)
- Multi-source validation for critical technical claims

**Scope Confirmed:** 2026-03-10

---

## Technology Stack Analysis

### Trunk Cache Busting (Working Correctly)

Trunk hashes asset filenames by default (`filehash = true`). The build output confirms this:

```
input-af2442d2cc9bb5b6.css
web-173eeb5ce251e765.js
web-173eeb5ce251e765_bg.wasm
```

Every new build produces different hashes. When `index.html` is refreshed, browsers fetch the new assets automatically. No configuration changes needed.

_Source: [Trunk Assets Documentation](https://trunkrs.dev/assets/), [Leptos #2005](https://github.com/leptos-rs/leptos/issues/2005)_

### GitHub Pages Cache Headers (Acceptable, Not Customizable)

GitHub Pages sets `Cache-Control: max-age=600` (10 minutes) on **all** files. This cannot be customized — GitHub Pages does not support `_headers` files, `.htaccess`, or any mechanism to set custom HTTP headers.

| File Type | Ideal Cache Policy | GitHub Pages Actual | Gap |
|-----------|-------------------|---------------------|-----|
| `index.html` | `no-cache` or short TTL | `max-age=600` | Minor — 10 min is acceptable |
| Hashed CSS/JS/WASM | `max-age=31536000` (immutable) | `max-age=600` | Missed optimization, but harmless |
| `sw.js` | `max-age=0` or `no-cache` | `max-age=600` | Minor — browsers check SW updates regardless |
| SoundFont / worklet | Long TTL | `max-age=600` | Missed optimization, but harmless |

Without the service worker, GitHub Pages' 10-minute TTL on `index.html` would be sufficient — users would get updates within 10 minutes of deployment.

_Source: [GitHub Community Discussion #11884](https://github.com/orgs/community/discussions/11884), [GitHub Discussion #49753](https://github.com/orgs/community/discussions/49753)_

### Non-Hashed Assets

These files are copied without content hashes:

- `soundfont/` directory including `synth_worklet.wasm` (via `copy-dir`)
- `GeneralUser-GS.sf2` (via `copy-file`)
- `sw.js` (via `copy-file`)

The SoundFont and worklet files change rarely, so the 10-min GitHub Pages TTL is adequate for them. The service worker file is a special case discussed below.

---

## Integration Patterns Analysis

### How the Components Interact

The update flow involves three layers:

```
User visits site
  → Browser checks sw.js (byte-for-byte comparison)
    → If sw.js unchanged: NO update cycle triggers
    → Old service worker continues controlling the page
      → Navigation requests: serves cached index.html (cache-first)
      → Asset requests: serves cached assets (cache-first)
      → User sees OLD version indefinitely
```

### Critical Issue: Service Worker Prevents Updates

The current `sw.js` has a well-documented anti-pattern that affects Angular, React, Svelte, and Preact apps alike:

**Problem 1 — `sw.js` never triggers an update:**
`CACHE_VERSION = 'peach-v1'` is hardcoded and the file is a static `copy-file`. Since the content never changes between builds, the browser's byte-for-byte comparison finds no difference → no `install` event → no cache refresh.

**Problem 2 — `index.html` is cached forever by the SW:**
The navigation handler uses cache-first:
```js
caches.match('./index.html').then(cached => cached || fetch(event.request))
```
Once cached, the service worker serves the old `index.html` forever. The old `index.html` references old hashed assets. Even though Trunk produced new hashed files, users never see them.

**Problem 3 — All sub-resources permanently cached:**
The fetch handler caches all responses under the static `CACHE_VERSION` key. Since the version never changes, old assets are never evicted.

_Source: [Workbox #1528](https://github.com/GoogleChrome/workbox/issues/1528), [Angular #27701](https://github.com/angular/angular/issues/27701), [Preact CLI #474](https://github.com/preactjs/preact-cli/issues/474), [CRA #7700](https://github.com/facebook/create-react-app/issues/7700)_

### Recommended Caching Strategy by Asset Type

Industry best practice for SPAs with service workers:

| Asset Type | Strategy | Rationale |
|------------|----------|-----------|
| `index.html` / navigation | **Network-first** | Always try fresh; fall back to cache for offline |
| Hashed assets (CSS/JS/WASM) | **Cache-first** | Hash in filename guarantees correctness; cache forever |
| Non-hashed assets (SF2, worklet) | **Stale-while-revalidate** | Serve fast from cache, refresh in background |
| `sw.js` | Must change on each build | Triggers the update lifecycle |

_Source: [MDN PWA Caching](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Guides/Caching), [Chrome Workbox Strategies](https://developer.chrome.com/docs/workbox/caching-strategies-overview), [web.dev Service Worker Lifecycle](https://web.dev/articles/service-worker-lifecycle)_

### How to Fix: Two Required Changes

**Fix 1 — Inject a build hash into `sw.js`:**
Add a `post_build` hook in `Trunk.toml` that replaces `CACHE_VERSION = 'peach-v1'` with a hash derived from the build output (e.g., hash of the WASM filename). This ensures every deploy produces a different `sw.js`, triggering the browser's update lifecycle.

Example approach:
```toml
[[hooks]]
stage = "post_build"
command = "sh"
command_arguments = ["-c", "HASH=$(ls dist/*.wasm | head -1 | sed 's/.*-\\([a-f0-9]*\\)_.*/\\1/'); sed -i \"s/peach-v1/peach-${HASH}/\" dist/sw.js"]
```

**Fix 2 — Use network-first for navigation requests:**
Change the service worker's navigation handler from cache-first to network-first:

```js
// BEFORE (broken): cache-first for index.html
caches.match('./index.html').then(cached => cached || fetch(event.request))

// AFTER (correct): network-first for index.html
fetch(event.request)
  .then(response => {
    const clone = response.clone();
    caches.open(CACHE_VERSION)
      .then(cache => cache.put('./index.html', clone));
    return response;
  })
  .catch(() => caches.match('./index.html'))
```

This ensures online users always get the latest `index.html` (with new hashed asset references), while offline users still get the cached version as a fallback.

**Combined effect:** When a new version deploys → `sw.js` has new content → browser detects update → installs new SW → new SW activates and clears old cache → navigation requests fetch fresh `index.html` → new hashed assets load.

---

## Architectural Patterns and Design

### Pattern: Network-First for Navigation

The recommended approach for `index.html` in SPAs with service workers. The service worker attempts a network fetch first, updates the cache with the response, and falls back to cache only when offline.

**Implementation:**

```js
self.addEventListener('fetch', (event) => {
  if (event.request.method !== 'GET') return;

  if (event.request.mode === 'navigate') {
    // Network-first: always try fresh index.html when online
    event.respondWith(
      fetch(event.request)
        .then(response => {
          const clone = response.clone();
          caches.open(CACHE_VERSION)
            .then(cache => cache.put('./index.html', clone));
          return response;
        })
        .catch(() => caches.match('./index.html'))
    );
    return;
  }

  // Cache-first for hashed assets (hash guarantees correctness)
  event.respondWith(
    caches.match(event.request).then(cached => {
      if (cached) return cached;
      return fetch(event.request).then(response => {
        if (!response || response.status !== 200 || response.type === 'opaque') {
          return response;
        }
        const clone = response.clone();
        caches.open(CACHE_VERSION)
          .then(cache => cache.put(event.request, clone))
          .catch(() => {});
        return response;
      });
    })
  );
});
```

_Source: [MDN PWA Caching](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Guides/Caching), [Chrome Workbox Strategies](https://developer.chrome.com/docs/workbox/caching-strategies-overview), [Network-first gist](https://gist.github.com/JMPerez/8ca8d5ffcc0cc45a8b4e1c279efd8a94)_

### Pattern: Build-Time Version Injection

Since `sw.js` must live at a fixed URL (service worker registration path cannot be hashed), the file content must change between builds to trigger the browser's update detection. The simplest approach: inject a hash derived from the build output.

**Trunk `post_build` hook approach:**

```toml
[[hooks]]
stage = "post_build"
command = "sh"
command_arguments = ["-c", "HASH=$(sha256sum dist/*.wasm | head -c16); sed -i '' \"s/peach-v1/peach-${HASH}/\" dist/sw.js"]
```

This extracts a hash from the WASM binary (which changes every build) and replaces the `CACHE_VERSION` placeholder. The browser detects that `sw.js` has changed → triggers `install` → new cache version → old caches cleaned up in `activate`.

**Cross-platform note:** `sed -i ''` (macOS) vs `sed -i` (Linux). In CI (ubuntu-latest), use `sed -i`. Locally on macOS, use `sed -i ''`. A portable alternative: use `perl -pi -e` which works on both.

**CI approach (in `.github/workflows/ci.yml`):**

```yaml
- name: Inject build hash into service worker
  run: |
    HASH=$(sha256sum dist/*.wasm | head -c16)
    sed -i "s/peach-v1/peach-${HASH}/" dist/sw.js
```

This is simpler than the Trunk hook and runs in the CI pipeline where the platform is known (Linux).

_Source: [Trunk build hooks](https://trunkrs.dev/), [Service Worker Lifecycle](https://web.dev/articles/service-worker-lifecycle), [Handling SW Updates](https://whatwebcando.today/articles/handling-service-worker-updates/)_

### Design Decision: Update UX

Three options for how users experience an update:

| Option | Behavior | Complexity | Recommendation |
|--------|----------|------------|----------------|
| **Silent update** | `skipWaiting()` + `clients.claim()` in new SW; user gets new version on next navigation | Low | **Recommended for peach-web** |
| **Reload prompt** | Show "New version available" toast, user clicks to reload | Medium | Better UX but requires Leptos UI integration |
| **Force reload** | `clients.claim()` triggers `controllerchange` → `location.reload()` | Low | Disruptive; not recommended |

The current `sw.js` already uses `skipWaiting()` + `clients.claim()`, which is the silent update pattern. This is appropriate for peach-web: the user gets the new version on their next page load or navigation within the SPA, with no interruption.

_Source: [Chrome: Handling SW updates with immediacy](https://developer.chrome.com/docs/workbox/handling-service-worker-updates), [Displaying "new version available"](https://deanhume.com/displaying-a-new-version-available-progressive-web-app/)_

### Summary: Required Changes

Only **two changes** are needed to fix the update problem:

1. **`sw.js`**: Change the navigation fetch handler from cache-first to network-first (code above)
2. **CI pipeline** (or Trunk hook): Inject a build hash into `CACHE_VERSION` so the browser detects `sw.js` has changed

Everything else in the current setup is correct:
- Trunk's hashed filenames work
- `skipWaiting()` + `clients.claim()` are already in place
- The `activate` handler already cleans up old caches
- The `404.html` copy for SPA routing is correct
- GitHub Pages' 10-min TTL is acceptable

---

## Implementation Approaches

### Change 1: Rewrite `sw.js` Fetch Handler

Replace the entire `fetch` event listener in `sw.js`:

```js
// sw.js — Service Worker for Peach ear training app
const CACHE_VERSION = 'peach-v1';  // Replaced at build time by CI

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_VERSION)
      .then(cache => cache.addAll(['./index.html']))
      .then(() => self.skipWaiting())
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys()
      .then(keys => Promise.all(
        keys.filter(k => k !== CACHE_VERSION).map(k => caches.delete(k))
      ))
      .then(() => self.clients.claim())
  );
});

self.addEventListener('fetch', (event) => {
  if (event.request.method !== 'GET') return;

  // Navigation requests: network-first with cache fallback (offline support)
  if (event.request.mode === 'navigate') {
    event.respondWith(
      fetch(event.request)
        .then(response => {
          const clone = response.clone();
          caches.open(CACHE_VERSION)
            .then(cache => cache.put('./index.html', clone));
          return response;
        })
        .catch(() => caches.match('./index.html'))
    );
    return;
  }

  // All other assets: cache-first with network fallback
  event.respondWith(
    caches.match(event.request).then(cached => {
      if (cached) return cached;
      return fetch(event.request).then(response => {
        if (!response || response.status !== 200 || response.type === 'opaque') {
          return response;
        }
        const clone = response.clone();
        caches.open(CACHE_VERSION)
          .then(cache => cache.put(event.request, clone))
          .catch(() => {});
        return response;
      });
    })
  );
});
```

**What changed:** Only the `navigate` branch of the fetch handler. The `install` and `activate` handlers remain identical. The comment on `CACHE_VERSION` clarifies it's a build-time placeholder.

### Change 2: CI Pipeline — Inject Build Hash

Add a step in `.github/workflows/ci.yml` between `trunk build` and `Upload Pages artifact`:

```yaml
- name: Inject build hash into service worker
  run: |
    HASH=$(sha256sum dist/*_bg.wasm | cut -c1-16)
    sed -i "s/peach-v1/peach-${HASH}/" dist/sw.js
    echo "Injected cache version: peach-${HASH}"
```

**Why `*_bg.wasm`:** This is the main app WASM binary produced by Trunk. Its hash changes on every code change. Using `sha256sum` of its content (not just the filename hash) ensures the version changes even if Trunk's filename hash algorithm changes.

**Why CI and not Trunk hook:** The CI runs on Linux where `sed -i` works without the macOS `''` quirk. A Trunk `post_build` hook would also work but requires cross-platform `sed` handling for local dev.

### Testing the Fix

**Local testing with `trunk serve`:**

1. Run `trunk serve` — note that locally `CACHE_VERSION` stays `peach-v1` (no CI injection), which is fine for development
2. Use Chrome DevTools → Application → Service Workers → check "Update on reload" during development
3. To test the actual update flow: build twice with `trunk build`, manually change `CACHE_VERSION` in `dist/sw.js`, and serve with a local HTTP server

**Production verification after deploy:**

1. Visit the deployed site, open DevTools → Application → Service Workers
2. Verify the active SW shows the new cache version (e.g., `peach-a1b2c3d4e5f6g7h8`)
3. Check Application → Cache Storage — should show the versioned cache name
4. Deploy a new version, revisit the site
5. Verify: SW status shows "waiting to activate" or has already activated with a new version
6. Navigation should serve the fresh `index.html` with new hashed asset URLs

_Source: [Chrome DevTools PWA debugging](https://developer.chrome.com/docs/devtools/progressive-web-apps), [SW debugging best practices](https://www.chromium.org/blink/serviceworker/service-worker-faq/), [Trunk configuration](https://trunkrs.dev/)_

### Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| First-time visitors with old SW cached | High (all current users) | Users stuck on old version until SW updates | The new `sw.js` with different content triggers immediate update cycle |
| `sha256sum` not available in CI | Very low | Build fails | `sha256sum` is standard on ubuntu-latest |
| `sed` regex doesn't match | Low | SW deploys without version, same as current bug | CI step echoes the injected version for verification |
| Network-first adds latency for navigation | Very low | ~50ms for `index.html` fetch | Negligible; `index.html` is tiny (~1KB) |
| Offline users don't get update | Expected | Users see cached version | Correct behavior — they get the update when back online |

### One-Time Migration for Existing Users

Current users have `peach-v1` cached. After deploying the fix:

1. Browser fetches `sw.js` on next visit (within GitHub Pages' 10-min TTL)
2. `sw.js` content has changed (new `CACHE_VERSION`) → browser triggers `install`
3. New SW calls `skipWaiting()` → immediately activates
4. `activate` handler deletes all caches where key !== new version → `peach-v1` cache deleted
5. Next navigation → network-first → fresh `index.html` → new hashed assets load
6. User is on the latest version

**No manual intervention needed.** All existing users will automatically migrate within ~10 minutes of their next visit.

---

## Technical Research Recommendations

### Implementation Roadmap

1. **Immediate (this sprint):** Apply the two changes (sw.js fetch handler + CI hash injection)
2. **Verify:** Deploy and confirm via DevTools that the update cycle works
3. **Optional future:** Consider a "New version available" toast if silent updates prove insufficient

### Success Metrics

- After deploy: new SW version visible in DevTools within 10 minutes of visit
- Navigation requests return fresh `index.html` (verify via Network tab — status 200, not from SW cache)
- Offline fallback still works (toggle offline in DevTools → navigation returns cached `index.html`)
- Old `peach-v1` cache is deleted after update

---

## Research Synthesis and Conclusion

### Audit Verdict

| Component | Verdict | Action |
|-----------|---------|--------|
| Trunk filename hashing | Working | None |
| GitHub Pages cache headers | Acceptable | None (not customizable) |
| CI build & deploy pipeline | Working | Add 1 step (hash injection) |
| `sw.js` — install/activate | Working | None |
| `sw.js` — fetch (navigation) | **Broken** | Switch to network-first |
| `sw.js` — fetch (assets) | Working | None |
| `sw.js` — version management | **Broken** | CI hash injection |
| SPA routing (404.html copy) | Working | None |
| Offline support | Working (will continue) | None |

### Key Finding

The app's caching architecture is sound — the single defect is the service worker's navigation fetch strategy combined with a static version string. This is a well-documented anti-pattern across the web development ecosystem (Angular, React, Svelte, Preact have all had identical bugs filed). The fix is minimal, low-risk, and self-migrating for existing users.

### Source Documentation

All technical claims were verified against current public sources:

- [GitHub Community Discussion #11884](https://github.com/orgs/community/discussions/11884) — GitHub Pages cache headers
- [GitHub Community Discussion #49753](https://github.com/orgs/community/discussions/49753) — GitHub Pages stale content
- [Trunk Assets Documentation](https://trunkrs.dev/assets/) — Trunk cache busting
- [Leptos #2005](https://github.com/leptos-rs/leptos/issues/2005) — WASM cache busting
- [MDN PWA Caching](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Guides/Caching) — Caching strategies
- [Chrome Workbox Strategies](https://developer.chrome.com/docs/workbox/caching-strategies-overview) — Network-first pattern
- [web.dev Service Worker Lifecycle](https://web.dev/articles/service-worker-lifecycle) — SW update detection
- [Workbox #1528](https://github.com/GoogleChrome/workbox/issues/1528) — Stale index.html after deploy
- [Angular #27701](https://github.com/angular/angular/issues/27701) — SW returning outdated index.html
- [Chrome DevTools PWA Debugging](https://developer.chrome.com/docs/devtools/progressive-web-apps) — Testing approach
- [Handling Service Worker Updates](https://whatwebcando.today/articles/handling-service-worker-updates/) — Update patterns

---

**Technical Research Completion Date:** 2026-03-10
**Source Verification:** All facts cited with current sources
**Confidence Level:** High — consistent findings across multiple authoritative sources
