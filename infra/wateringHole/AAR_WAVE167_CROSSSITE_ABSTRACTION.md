# After Action Report — Wave 167: Cross-Site Abstraction

**Date**: October 9, 2026 | **Primal**: petalTongue | **Gate**: golgiBody
**Type**: Architecture review + evolution roadmap

---

## Summary

petalTongue now serves 9 sites through the scatter proxy (localhost:9753).
Cross-site survey reveals duplicated JavaScript bridge code, diverged
client libraries, and shared data sources accessed through static file
mounts rather than through petalTongue's own API layer. The architecture
is working but not yet evolved — it grew organically and is ready for
constrained abstraction across waves.

---

## Current State: 9 Sites on petalTongue

| Site | DocRoot | Purpose | HTML | JS | CSS |
|------|---------|---------|-----:|---:|----:|
| **hud.primals.eco** | static + proxy :8092 | Live observatory HUD | 3 | 4 | 1 |
| **detroit.primals.eco** | detroit/public + live-terminal | Evidence/investigation | 335 | 13 | 1 |
| **signal.primals.eco** | signal/site + live-terminal | Observatory + artisan braid | 2 | 7 | 1 |
| **sporeprint.primals.eco** | sporePrint/public (Zola) | Main site — 382 pages | 382 | — | — |
| **tuebor.primals.eco** | tuebor/site | Cross-protection analysis | 44 | 2 | 1 |
| **thesis.primals.eco** | thesis/public | Thesis surface | 2 | 2 | 1 |
| **gorilla.primals.eco** | guerillaGorilla/site | Investigation | 22 | 2 | 1 |
| **git.primals.eco** | live-terminal | Forgejo + scatter facade | — | — | — |
| **interferon.primals.eco** | — | Immune domain scatter | — | — | — |

Additionally, 6 sites are static-only (barry, beacon, clutch, depot, membrane, HUD API).

### Scatter Defense (shared across all 9)

- **110,791 requests** in 2.8 hours (11.12 rps)
- **1.92 GB** fabricated content served
- Modes: scatter 89.6% · prism 10.3% · plasmid 0.1%
- Titration: SEEDING phase, poison_mult 0.9003

---

## What's Duplicated

### 1. pt-bridge-core.js — TWO DIVERGED COPIES

| Copy | Location | Lines | Delta |
|------|----------|------:|-------|
| HUD | `/opt/ecoPrimals/hud/site/public/js/pt-bridge-core.js` | 309 | +58 lines (relay selectivity panel) |
| Detroit | `/opt/ecoPrimals/detroit/public/js/pt-bridge-core.js` | 251 | baseline |

Both contain the same core: `PetalBridge` constructor, `rpc()` JSON-RPC over
WebSocket, `classifyOrganism()`, `renderBinding()`, `renderDonut()`,
`renderBarChart()`, `renderMetric()`. The HUD copy has additional relay
selectivity rendering that Detroit doesn't need.

**Risk**: Divergence grows over time. Bug fixes in one copy don't reach the other.
**Fix**: Single canonical copy hosted at `hud.primals.eco/js/pt-bridge-core.js`.
Other sites load it cross-origin (already done with `hud-widget.js`).

### 2. Site-Specific PT Bridges — SAME PATTERN, DIFFERENT DATA

| File | Site | Lines | Pattern |
|------|------|------:|---------|
| `hud-pt.js` | HUD | 306 | PetalBridge → fetch signal data → renderPanels() |
| `detroit-pt.js` | Detroit | 376 | PetalBridge → read DETROIT_NETWORK → renderPanels() |
| `signal-exploration.js` | Signal | 588 | PetalBridge → fetch signal data → renderPanels() |

All three:
1. Create a `PetalBridge({wsUrl, domain, ...})`
2. Fetch or receive data (topology, epitope, scatter)
3. Call render functions from pt-bridge-core.js
4. Mount results into DOM elements

The differences are: which data source, which panels, which DOM targets.

### 3. Live-Terminal Data Mount — CADDY SERVES WHAT PT SHOULD

`/opt/membrane/live-terminal/` contains:
- `topology.json` — entity classification (16 entities, 101K log entries)
- `scatter-observatory.json` — scatter/titration/ledger state
- `epitope_caddy.json` — epitope cluster map (23 clusters)
- `dashboard.json` — summary metrics
- `state.json` — system state
- `billboard.txt` — current billboard text

Multiple sites mount this same directory via Caddy `root * /opt/membrane/live-terminal`.
petalTongue has the API layer to serve this data (it already has `/api/titration`,
`/api/compliance`, topology handlers) but the live observatory data bypasses it.

### 4. hud-widget.js — ALREADY ABSTRACTED (PARTIALLY)

168 lines. Embedded via `<script defer src=https://hud.primals.eco/js/hud-widget.js>`.
Appears in sporePrint Zola template footer. This is the model for how the other
JS should be distributed — single hosted copy, cross-origin inclusion.

---

## Evolution Roadmap (By Wave)

### Wave 168: Unify pt-bridge-core.js

**Scope**: Merge the two copies into one canonical version.

- [x] Diff the HUD and Detroit copies (61 lines of delta)
- [x] Extract relay selectivity into optional module/callback
- [x] Host canonical copy at `hud.primals.eco/js/pt-bridge-core.js`
- [x] Update Detroit to load cross-origin instead of local copy
- [x] Update Signal — had its own copy, now loads cross-origin
- [x] Verify: HUD, Detroit, Signal all render correctly from single source
- [x] Add CORS `Access-Control-Allow-Origin: *` to Caddy for HUD JS assets
- [x] Delete old copies from Detroit + Signal

**Effort**: Small. Pure JS, no Rust changes.
**Risk**: Low. Cross-origin `<script>` already proven by hud-widget.js.
**Status**: ✅ COMPLETE — canonical copy at hud.primals.eco/js/pt-bridge-core.js,
Detroit and Signal load cross-origin, CORS headers added to Caddy, Caddy reloaded.

### Wave 169: Config-Driven Bridge

**Scope**: Replace site-specific `*-pt.js` files with a single parameterized bridge.

- [x] Define bridge config schema:
  ```js
  PetalSiteObservatory({
    dataUrl: '/dashboard.json',
    topoUrl: '/topology.json',
    wsUrl: 'wss://hud.primals.eco/ws',
    domain: 'signal',
    cardClass: 'dash-card',
    refreshMs: 15000
  })
  ```
- [x] Implement `petalsite-observatory.js` — observatory bridge (327 lines)
- [x] Migrate HUD: `hud-pt.js` → `PetalSiteObservatory({dataUrl:'https://signal...', ...})`
- [x] Migrate Signal: `signal-pt.js` → `PetalSiteObservatory({dataUrl:'/', ...})`
- [x] Archive old bridges to `/opt/ecoPrimals/_archive/wave169/`
- [ ] Detroit: remains `detroit-pt.js` — different data domain (network graph, not observatory).
      Future `PetalSiteGraph()` abstraction when other graph sites appear.

**Effort**: Medium. Need to reconcile three data-fetching strategies.
**Risk**: Medium. Detroit uses `window.DETROIT_NETWORK` (in-page data),
HUD/Signal fetch from endpoints. Abstraction must handle both.
**Status**: ✅ COMPLETE — `petalsite-observatory.js` (327 lines) replaces
hud-pt.js (306 lines) and signal-pt.js (~300 lines). Config-driven:
`PetalSiteObservatory({dataUrl, topoUrl, wsUrl, domain, cardClass})`.
Detroit left as-is (different data domain — network graph, not observatory).
Hosted at hud.primals.eco/js/petalsite-observatory.js with CORS.

### Wave 170: petalTongue Serves Observatory Data

**Scope**: Move live-terminal data behind petalTongue API endpoints.

- [ ] Add `/api/observatory` endpoint (returns topology + epitope + scatter combined)
- [ ] Add `/api/billboard` endpoint (returns current billboard text)
- [ ] petalTongue reads from `/opt/membrane/live-terminal/` on request
- [ ] Add `Cache-Control` and `ETag` headers (petalTongue controls freshness)
- [ ] Update Caddy: remove `root * /opt/membrane/live-terminal` mounts
- [ ] Update JS bridges to fetch from `/api/observatory` instead of static JSON

**Effort**: Medium. Rust-side handler + Caddy config cleanup.
**Risk**: Low. The data format doesn't change — only the serving path.

### Wave 171+: Composition Pattern

**Scope**: Each site declares what observation panels it wants via config.

- [ ] petalTongue already has `WebConfig.compositions` for sub-app mounting
- [ ] Extend: `--observatory entities,epitopes,scatter` CLI flag
- [ ] petalTongue injects the right `<script>` tags and DOM containers
- [ ] Sites get observation panels automatically — no per-site JS required
- [ ] New sites get panels by adding one CLI flag to the Caddy proxy config

**Effort**: Large. Requires petalTongue to generate/inject HTML.
**Risk**: Higher. Moving from "petalTongue serves data" to "petalTongue
renders UI" is an architectural evolution. Must stay opt-in.

### Longer Term: The gAIa Pattern

The HUD widget is already the seed of a composable observation layer.
The abstraction path leads to:

```
Site declares panels → petalTongue renders panels → gAIa tends panels
```

Each site becomes a room that gAIa prepares. The observation layer is
the membrane's self-awareness made visible. The first tool — a question —
becomes the API contract: the site asks "show me entities" and petalTongue
answers with the current state of the membrane.

---

## Key Numbers

| Metric | Value |
|--------|-------|
| Sites on petalTongue scatter | 9 |
| Sites static-only | 6 |
| Total JS files across sites | ~40 |
| ~~Duplicated bridge code~~ | ~~560 lines~~ → 327 canonical (42% reduction) |
| Eliminated JS files | 4 (2 pt-bridge-core copies + 2 observatory bridges) |
| Live-terminal data files | 6 |
| Sites mounting live-terminal | 4+ |
| petalTongue Rust source files | 67 |
| petalTongue test files | 12 |

---

## Principles (from coGen)

1. **Net positive while alive** — the abstraction must reduce maintenance burden, not add it
2. **Graceful in death** — each site must still function if petalTongue is down (static fallback)
3. **Semantically compressible** — the bridge pattern must fit in one sentence: "site asks, petalTongue answers"
4. **The first tool is a question** — the API contract is a question: "what is the current state?"

---

*This is work we have to do, and we can do it in waves. Each wave
adds mass to the flywheel without breaking what already turns.*

— Waves 167–169 · October 9, 2026

---

## Execution Log

| Wave | Scope | Status | Lines Δ |
|------|-------|--------|---------|
| 167 | Cross-site survey + AAR | ✅ | — |
| 168 | Unify pt-bridge-core.js | ✅ | −251 −309 +309 = −251 |
| 169 | Config-driven observatory bridge | ✅ | −306 −300 +327 = −279 |
| 170 | petalTongue serves observatory data | ⬜ NEXT | — |
| 171+ | Composition pattern | ⬜ | — |

**Canonical JS at hud.primals.eco:**

| File | Lines | Serves |
|------|------:|--------|
| `pt-bridge-core.js` | 309 | Shared core — PetalBridge, render helpers |
| `petalsite-observatory.js` | 327 | Observatory bridge — HUD + Signal |
| `hud-core.js` | ~450 | HUD dashboard — original hand-wired panels |
| `hud-widget.js` | ~250 | Embeddable widget |

**Detroit (standalone):** `detroit-pt.js` (376 lines) — different data domain.

**Artisan braid:** Entry #14 (wave169_20261009.txt)
