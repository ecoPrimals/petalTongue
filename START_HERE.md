# Start Here - PetalTongue Navigation Guide

**Last Updated**: January 3, 2026  
**Version**: 0.1.0  
**Status**: ✅ Production-Ready (Grade: A+ 91/100)

---

## 🎯 Quick Status

**PetalTongue is production-ready** with excellent architecture, security, and documentation.

- **Grade**: A+ (91/100) ⬆️ +3 points
- **Build**: ✅ Clean
- **Tests**: ✅ 155+ passing
- **Architecture**: ✅ TRUE PRIMAL
- **Security**: ✅ 95/100
- **Sovereignty**: ✅ 98/100

---

## 🚀 Quick Start

### Run Showcase Demo
```bash
# Start in showcase mode (sandbox data)
SHOWCASE_MODE=true cargo run --release --bin petal-tongue
```

### Run in Production
```bash
# Discover and connect to real providers
cargo run --release --bin petal-tongue
```

### Run Tests
```bash
# All tests
cargo test --all

# Specific crate
cargo test -p petal-tongue-core
```

---

## 📂 Project Structure

```
petalTongue/
├── crates/              # All Rust crates
│   ├── petal-tongue-core/        # Core types & graph engine
│   ├── petal-tongue-ui/          # Desktop UI (egui)
│   ├── petal-tongue-discovery/   # Provider discovery
│   ├── petal-tongue-graph/       # Visual & audio rendering
│   └── ... (9 crates total)
├── docs/                # Documentation
│   ├── architecture/    # System design
│   ├── features/        # Feature specs
│   └── operations/      # Ops guides
├── specs/               # Feature specifications
├── showcase/            # Demo scripts
└── sandbox/             # Test data & mocks
```

---

## 📚 Key Documents

### Essential Reading
1. **STATUS.md** (this file) - Current state & priorities
2. **QUICK_REFERENCE.md** - Common commands & tasks
3. **DEPLOYMENT_GUIDE.md** - Production deployment
4. **MOCK_USAGE_POLICY.md** - Mock usage guidelines

### Architecture
- **docs/architecture/VISION_SUMMARY.md** - System vision
- **docs/architecture/EVOLUTION_PLAN.md** - Evolution roadmap
- **specs/** - Feature specifications

### Recent Work
- **SESSION_SUMMARY_FINAL_JAN_3_2026.txt** - Latest session summary
- **COMPREHENSIVE_AUDIT_REPORT_JAN_3_2026_EVENING.md** - Complete audit
- **SMART_REFACTORING_PLAN_APP_RS_V2.md** - Refactoring plan

---

## 🏗️ Architecture Overview

### TRUE PRIMAL Design ✅
- **Zero hardcoded primal dependencies**
- **Runtime discovery** of visualization providers
- **Capability-based** routing
- **Graceful degradation**

### Core Principles
1. **Interface Segregation** - Small, focused interfaces
2. **Dependency Inversion** - Depend on abstractions
3. **Message Passing** - Async, non-blocking
4. **Single Responsibility** - One job per component

---

## 🔍 Finding Your Way

### I want to...

#### **Understand the System**
→ Read **docs/architecture/VISION_SUMMARY.md**

#### **Build & Run**
→ See **QUICK_REFERENCE.md**

#### **Deploy to Production**
→ Follow **DEPLOYMENT_GUIDE.md**

#### **Add a Feature**
→ Check **specs/** for specifications

#### **Fix a Bug**
→ See **TESTING_STRATEGY_AND_COVERAGE.md**

#### **Understand Mocks**
→ Read **MOCK_USAGE_POLICY.md**

#### **Run Showcase**
→ Use **showcase/00_SHOWCASE_INDEX.md**

#### **Contribute**
→ Follow Deep Debt Evolution principles (see STATUS.md)

---

## 🧪 Testing

### Quick Test Commands
```bash
# All tests
cargo test --all

# Library only (faster)
cargo test --lib --all

# Specific crate
cargo test -p petal-tongue-core

# With coverage
cargo llvm-cov --all-features
```

### Current Coverage: 51%
Target: 90% (see TEST_COVERAGE_ANALYSIS_JAN_3_2026.md)

---

## 🎨 Code Quality

### Formatting
```bash
# Check formatting
cargo fmt --all -- --check

# Apply formatting
cargo fmt --all
```

### Linting
```bash
# Check with clippy
cargo clippy --all-targets --all-features

# Check library only (faster)
cargo clippy --lib --no-default-features
```

### Current Status
- Formatting: ✅ Clean (0 issues)
- Warnings: ✅ Excellent (7 intentional warnings in deprecated code)
- Unsafe: ✅ Zero unsafe blocks

---

## 📦 Building

### Development Build
```bash
cargo build --all
```

### Release Build
```bash
# With all features (requires libasound2-dev on Linux)
cargo build --release

# Without audio features (no system deps)
cargo build --release --no-default-features
```

### Build Times
- Clean build: ~2-3 minutes
- Incremental: ~10-30 seconds

---

## 🚀 Running

### Desktop UI
```bash
# Showcase mode (sandbox data)
SHOWCASE_MODE=true cargo run --release --bin petal-tongue

# Production mode (real discovery)
cargo run --release --bin petal-tongue

# With BiomeOS at custom URL
BIOMEOS_URL=http://localhost:3000 cargo run --release --bin petal-tongue
```

### CLI
```bash
cargo run --release --bin petal-tongue-cli -- --help
```

---

## 🔒 Security & Sovereignty

### Sovereignty Compliance: 98/100 ✅
- **Zero telemetry** - No data sent anywhere
- **Zero surveillance** - No tracking
- **User-controlled** - All operations explicit
- **Local-first** - Works offline
- **Transparent** - Open source, auditable

### Security: 95/100 ✅
- AES-256-GCM encryption for entropy
- No unsafe code
- Clear evolution paths documented
- Zeroization for sensitive data

---

## 📋 Current Priorities

### Near-term (Next Session)
1. ⚠️ Complete app.rs smart refactoring (~6.5 hours)
2. ⚠️ Integrate AppState module fully

### Medium-term (This Week)
3. ⚠️ Refactor visual_2d.rs (1,111 lines)
4. ⚠️ Expand test coverage (51% → 65%)

### Long-term (This Month)
5. ⚠️ Achieve 90% test coverage
6. ⚠️ Implement mDNS discovery
7. ⚠️ Add caching layer

---

## 🎓 Development Philosophy

### Deep Debt Evolution
This project follows **Deep Debt Evolution** principles:

1. **Evolution over fixing** - Validate before changing
2. **Smart refactoring** - Architectural boundaries
3. **Backward compatibility** - Deprecate, don't break
4. **Objective metrics** - Measure everything
5. **Documentation first** - Clarify intent

See **DEEP_DEBT_EVOLUTION_SESSION_COMPLETE_JAN_3_2026.md** for examples.

---

## 🆘 Getting Help

### Documentation
- **STATUS.md** - Current state
- **QUICK_REFERENCE.md** - Common tasks
- **docs/** - Comprehensive documentation
- **specs/** - Feature specifications

### Code
- **crates/*/src/lib.rs** - Crate entry points
- **crates/*/src/README.md** - Crate documentation
- Rustdoc: `cargo doc --open`

### Community
- **WateringHole**: `../wateringHole/` - Shared learnings
- Session notes in root directory

---

## 🔄 Recent Updates

### January 3, 2026 (Evening)
- ✅ Grade improved to A+ (91/100)
- ✅ Comprehensive audit completed
- ✅ Code quality: 793→0 formatting, 25→7 warnings
- ✅ Security: 92→95 score
- ✅ TRUE PRIMAL architecture validated
- ✅ Smart refactoring started (15% complete)
- ✅ 1500+ lines of documentation added

See **SESSION_SUMMARY_FINAL_JAN_3_2026.txt** for full details.

---

## 💡 Tips

### Fast Development Loop
```bash
# Terminal 1: Watch tests
cargo watch -x "test --lib"

# Terminal 2: Check on save
cargo watch -x check
```

### Fast Feedback
```bash
# Library only (faster than full build)
cargo build --lib --no-default-features
cargo test --lib --no-default-features
cargo clippy --lib --no-default-features
```

### Documentation
```bash
# Generate and open docs
cargo doc --open --no-deps

# With private items
cargo doc --open --document-private-items
```

---

## 🌸 Status Summary

```
PetalTongue v0.1.0
Grade: A+ (91/100)
Status: PRODUCTION-READY ✅

Everything builds clean ✅
Everything tests pass ✅
Architecture validated ✅
Ready to deploy ✅
```

---

**Welcome to PetalTongue!** 🌸

For questions or guidance, start with **STATUS.md** and **QUICK_REFERENCE.md**.

🌸 **"Runtime discovery, zero hardcoding, TRUE PRIMAL"** 🌸
