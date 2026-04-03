# sporePrint Content Delivery Specification

**Status**: Specification v0.1 — evolution target
**Date**: April 3, 2026
**Primal**: petalTongue (Universal User Interface)
**Upstream**: sporePrint (primals.eco) — Zola site, Markdown + TOML front matter
**Standard**: `wateringHole/LINK_INTEGRITY_STANDARD.md`

---

## Purpose

sporePrint is the public record and verification portal for ecoPrimals. It
contains 49+ pages of structured scientific documentation rendered by Zola
(a Rust static site generator) at primals.eco. All content is Markdown with
TOML front matter — human-readable, machine-parseable, link-validated.

This specification defines how petalTongue consumes, indexes, and delivers
sporePrint content across all modalities (visual, audio, conversational,
tactile) while preserving the dead-end free guarantee established by the
Link Integrity Standard.

---

## Content Structure

### Source Format

Every sporePrint page is a Markdown file with TOML front matter:

```toml
+++
title = "Anderson Localization as QS Null Hypothesis"
description = "Physics x Microbiology — 3,100+ checks"
date = 2026-03-17

[extra]
paper_number = 1
domain = "Physics x Microbiology"
status = "Validated"
+++

# Full markdown content follows...
```

### Sections

| Section | Path | Content Type |
|---------|------|-------------|
| Landing | `content/_index.md` | Ecosystem overview, verification paths |
| Audience | `content/audience/` | Role-specific guides (5 pages) |
| Science | `content/science/` | baseCamp papers (25+ pages) |
| Architecture | `content/architecture/` | Ecosystem structure docs |
| Methodology | `content/methodology/` | How it was built |
| Technical | `content/technical/` | Hardware, grants, pipelines |
| guideStone | `content/guidestone/` | Verification class, deployment artifact |

### Search Index

Zola generates `search_index.en.js` (Elasticlunr) containing title,
description, path, and content for every page. petalTongue can consume
this index directly for conversational search.

---

## Delivery Modalities

### Visual (Desktop / Web)

Primary delivery: Zola renders HTML at primals.eco. petalTongue extends
this with:

- **LiveSpore View** — embed sporePrint content in petalTongue's scene
  graph as navigable panels
- **Grammar of Graphics** — render science paper data (check counts,
  validation matrices, cross-spring evidence) as interactive charts
- **Linked Selection** — clicking a paper in the science index
  highlights related springs, primals, and validation results across panels

### Conversational (Audio / Chat)

petalTongue parses the TOML front matter to build a conversational index:

```
User: "What papers are validated?"
petalTongue: [queries front matter where status = "Validated"]
  "25 papers are validated. The headline result is Paper 01,
   Anderson Localization as QS Null Hypothesis, with 3,100+
   checks in Physics times Microbiology..."
```

The TOML metadata provides structured answers without natural language
parsing of the full document. The `description` field serves as the
spoken summary. The `[extra]` fields enable faceted queries (by domain,
status, paper number).

### Tactile (Braille / Haptic)

The Markdown source is the accessible representation. petalTongue strips
HTML rendering artifacts and delivers the structured text with:

- Heading hierarchy preserved (H1 = document, H2 = section, H3 = subsection)
- Table data linearized into key-value pairs
- Code blocks announced and delimited
- Links announced with both label and destination
  (dead-end free: all links validated at build time)

### Headless (API / Agent)

petalTongue's headless mode serves sporePrint content as structured JSON
via JSON-RPC:

```json
{
  "method": "content.query",
  "params": {
    "section": "science",
    "filter": { "extra.status": "Validated" },
    "fields": ["title", "description", "extra.paper_number", "extra.domain"]
  }
}
```

AI agents querying the ecosystem documentation receive typed, structured
responses rather than scraping HTML. The TOML front matter is the API
contract.

---

## Link Integrity Inheritance

sporePrint's Link Integrity Standard guarantees:
- Internal links validated at `zola build` time
- External links validated by `zola check` in CI
- Broken links block deployment

petalTongue inherits this guarantee: every link in the content it
delivers has been verified. When petalTongue renders a link in any
modality (spoken URL, clickable element, braille reference), it can
trust the target exists.

This is the **dead-end free infranet** property: the trust chain
flows from Zola's build-time validation through petalTongue's runtime
delivery to the user's modality. No link encountered by any user
through any interface leads to a dead end.

---

## Implementation Phases

### Phase 0: Index Consumption (Current)

petalTongue can already consume the Elasticlunr search index and TOML
front matter via file reads. No Zola dependency. The content directory
is the interface.

### Phase 1: LiveSpore View

Embed sporePrint sections as navigable panels in petalTongue's desktop
and TUI interfaces. The Grammar of Graphics engine renders data tables
from the science papers as charts.

### Phase 2: Conversational Navigation

petalTongue's interaction engine accepts natural-language queries about
the ecosystem. Responses are composed from TOML metadata + Markdown
content sections. Squirrel (AI primal) can augment with contextual
understanding.

### Phase 3: Full Multimodal

Every modality in the UUI specification (visual, audio, tactile, haptic,
web, headless) has a sporePrint content adapter. A blind user navigating
the ecosystem via audio receives the same information as a sighted user
browsing primals.eco — different modality, same content, same link
integrity.

---

## Relationship to Other Specs

| Spec | Relationship |
|------|-------------|
| `UNIVERSAL_USER_INTERFACE_SPECIFICATION.md` | sporePrint content is a first-class input source for the UUI |
| `PRIMAL_MULTIMODAL_RENDERING_SPECIFICATION.md` | Markdown → modality compilation follows the multimodal pipeline |
| `BIDIRECTIONAL_UUI_ARCHITECTURE.md` | sporePrint is read-only input; future evolution: petalTongue writes annotations back |
| `GRAMMAR_OF_GRAPHICS_ARCHITECTURE.md` | Science paper data (tables, check counts) compiles to grammar expressions |
| `INTERACTION_ENGINE_ARCHITECTURE.md` | Conversational navigation uses the interaction engine |

---

*The content is the source of truth. The modality is the interface.
The link integrity is the guarantee. petalTongue delivers all three.*
