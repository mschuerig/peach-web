---
title: 'Fix service worker cache busting for app updates'
slug: 'fix-sw-cache-busting'
created: '2026-03-10'
status: 'completed'
stepsCompleted: [1, 2, 3, 4]
tech_stack: ['JavaScript (sw.js)', 'GitHub Actions YAML']
files_to_modify: ['sw.js', '.github/workflows/ci.yml']
code_patterns: ['Vanilla JS with Promise chains (no async/await)', 'Standard GitHub Actions workflow steps']
test_patterns: ['Manual verification via Chrome DevTools (Application → Service Workers / Cache Storage)', 'No automated SW tests exist']
---

# Tech-Spec: Fix service worker cache busting for app updates

**Created:** 2026-03-10

## Overview

### Problem Statement

The service worker's static `CACHE_VERSION = 'peach-v1'` never changes between builds, so the browser never detects a new service worker version and never triggers an update cycle. Additionally, the navigation fetch handler uses a cache-first strategy for `index.html`, meaning once cached, users are served the old `index.html` (with old hashed asset references) indefinitely — even when online.

### Solution

Two targeted changes: (1) switch the navigation fetch handler from cache-first to network-first so online users always get fresh `index.html`, and (2) add a CI pipeline step that injects a build-derived hash into `CACHE_VERSION` so the browser detects `sw.js` has changed on each deploy.

### Scope

**In Scope:**

- Rewrite `sw.js` navigation fetch handler to network-first with offline cache fallback
- Add CI step to inject WASM-derived hash into `CACHE_VERSION` after `trunk build`

**Out of Scope:**

- "New version available" UI toast/prompt
- Stale-while-revalidate for non-hashed assets (SoundFont, worklet)
- Custom cache headers (not possible on GitHub Pages)

## Context for Development

### Codebase Patterns

- `sw.js` is a plain JavaScript file at project root, copied to `dist/` by Trunk via `<link data-trunk rel="copy-file" href="sw.js" />` in `index.html`
- Service worker is registered inline in `index.html` body: `navigator.serviceWorker.register('./sw.js')`
- `install` handler pre-caches `./index.html` and calls `self.skipWaiting()`
- `activate` handler deletes old caches by version key and calls `self.clients.claim()`
- CI deploys via GitHub Actions using `actions/deploy-pages@v4`

### Files to Reference

| File | Purpose |
| ---- | ------- |
| `sw.js` | Service worker — navigation and asset caching |
| `.github/workflows/ci.yml` | CI pipeline — build and deploy to GitHub Pages |
| `index.html` | Trunk source HTML — registers sw.js |
| `docs/planning-artifacts/research/technical-github-pages-caching-research-2026-03-10.md` | Full research report with analysis and citations |

### Technical Decisions

- **Network-first for navigation, not stale-while-revalidate**: Network-first is simpler and guarantees online users always get the latest `index.html`. The ~50ms latency for fetching the tiny HTML shell is negligible.
- **CI hash injection, not Trunk post_build hook**: CI runs on Linux where `sed -i` works without macOS quirks. Simpler and more reliable than a cross-platform Trunk hook.
- **Silent update (existing pattern)**: `skipWaiting()` + `clients.claim()` are already in place. No UI changes needed.

## Implementation Plan

### Tasks

- [x] Task 1: Rewrite `sw.js` navigation fetch handler
  - File: `sw.js`
  - Action: Replace the cache-first navigation handler (lines 31-36) with network-first. Change:
    ```js
    // CURRENT (cache-first — broken)
    if (event.request.mode === 'navigate') {
      event.respondWith(
        caches.match('./index.html').then(cached => cached || fetch(event.request))
      );
      return;
    }
    ```
    To:
    ```js
    // NEW (network-first — correct)
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
    ```
  - Notes: `install` and `activate` handlers remain unchanged. The asset fetch handler (cache-first for hashed assets) also remains unchanged. Add a comment on `CACHE_VERSION` line: `// Replaced at build time by CI`

- [x] Task 2: Add CI step to inject build hash into `CACHE_VERSION`
  - File: `.github/workflows/ci.yml`
  - Action: Add a new step between "Copy index.html to 404.html for SPA routing" (line 82) and "Upload Pages artifact" (line 84):
    ```yaml
    - name: Inject build hash into service worker
      run: |
        HASH=$(sha256sum dist/*_bg.wasm | cut -c1-16)
        sed -i "s/peach-v1/peach-${HASH}/" dist/sw.js
        echo "Injected cache version: peach-${HASH}"
    ```
  - Notes: Uses `sha256sum` of the WASM binary content (not the Trunk filename hash) so the version changes even if Trunk's hashing algorithm changes. The `echo` line provides CI log visibility for debugging. `sha256sum` and `sed -i` are standard on `ubuntu-latest`.

### Acceptance Criteria

- [ ] AC 1: Given a user is online and a new version has been deployed, when they visit the app, then their navigation request fetches `index.html` from the network (not from SW cache), and the page loads the latest hashed assets.
- [ ] AC 2: Given a user is offline, when they navigate within the app, then the service worker serves the cached `index.html` and the app remains functional with cached assets.
- [ ] AC 3: Given the CI pipeline runs `trunk build --release`, when the "Inject build hash" step executes, then `dist/sw.js` contains a `CACHE_VERSION` value derived from the WASM binary hash (not `peach-v1`), and the CI log outputs the injected version string.
- [ ] AC 4: Given a user has the old `peach-v1` service worker cached, when they visit the site after the fix is deployed, then the browser detects `sw.js` has changed, installs the new service worker, and the `activate` handler deletes the old `peach-v1` cache.
- [ ] AC 5: Given a new service worker is installed, when it activates, then all caches except the current version are deleted (existing `activate` behavior preserved).

## Additional Context

### Dependencies

None — no new packages or crates required. `sha256sum` and `sed` are pre-installed on `ubuntu-latest`.

### Testing Strategy

**Manual verification (post-deploy):**

1. Open Chrome DevTools → Application → Service Workers
2. Confirm the active SW shows a versioned cache name (e.g., `peach-a1b2c3d4e5f6g7h8`), not `peach-v1`
3. Check Application → Cache Storage — should show the versioned cache key
4. Open Network tab, reload — verify `index.html` returns status 200 from network (not `(ServiceWorker)` in Size column)
5. Toggle "Offline" in DevTools → navigate within app → confirm cached `index.html` is served
6. Deploy a second build, revisit — confirm SW updates to new version and old cache is deleted

**CI verification:**

7. Check GitHub Actions log for the "Inject build hash" step — should output `Injected cache version: peach-<hash>`

### Notes

- Existing users will auto-migrate within ~10 minutes of their next visit after deploy (GitHub Pages' `max-age=600` TTL on `sw.js`)
- GitHub Pages sets `Cache-Control: max-age=600` on all files (not customizable)
- Trunk already hashes CSS/JS/WASM filenames — that part works correctly
- The `install` handler's `self.skipWaiting()` and `activate` handler's `self.clients.claim()` ensure immediate activation — no "waiting" state

## Review Notes

- Adversarial review completed
- Findings: 10 total, 5 fixed (F1/F10, F2, F3, F4, F6), 5 skipped (F5 noise, F7/F8 out of scope, F9 pre-existing)
- Resolution approach: auto-fix
