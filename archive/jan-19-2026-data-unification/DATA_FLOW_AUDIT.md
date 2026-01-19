# 🔍 Data Flow Audit - Finding Strays

**Date**: January 19, 2026  
**Goal**: Verify ALL data flows through DataService  
**Status**: 🚧 IN PROGRESS

---

## 🎯 Audit Strategy

### What We're Looking For

**Strays** - Code that fetches data WITHOUT using DataService:
- Direct `biomeos_client.discover_primals()` calls
- Direct `neural_api.get_primals()` calls
- Direct `discovery_provider.get_topology()` calls
- Any data fetching in UI modes

### Where to Look

1. **src/** - UniBin mode implementations
2. **crates/petal-tongue-ui/src/** - GUI implementation
3. **crates/petal-tongue-tui/src/** - TUI implementation
4. **All *_mode.rs files**

---

## 📊 Findings

### ✅ Already Using DataService

- `src/web_mode.rs` - ✅ Uses DataService
- `src/data_service.rs` - ✅ Single source of truth

### 🔍 Need to Check

- `src/ui_mode.rs` - ???
- `src/tui_mode.rs` - ???
- `src/headless_mode.rs` - ???
- `src/cli_mode.rs` - ???
- `crates/petal-tongue-ui/src/app.rs` - ???

---

## 🚨 Strays Found

(Will populate as we audit)

---

## ✅ Migration Plan

For each stray:
1. Remove direct data fetching
2. Add DataService parameter
3. Call `data_service.snapshot()` or `data_service.graph()`
4. Test

---

**Status**: Starting audit...

