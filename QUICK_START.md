# 🚀 Quick Start - petalTongue

**Last Updated**: December 25, 2025  
**Status**: Production Ready (95/100)

---

## ⚡ **Fastest Start**

```bash
# Run petalTongue UI
cargo run --release -p petal-tongue-ui

# Run BingoCube demo
cd bingoCube/demos && cargo run --release
```

---

## 📦 **Installation**

### Prerequisites:
- Rust 1.75+ (2021 edition or later)
- Cargo
- Linux/macOS/Windows

### Build:
```bash
# Clone (if needed)
cd /path/to/petalTongue

# Build everything
cargo build --all --release

# Test everything
cargo test --all
```

---

## 🎯 **Common Commands**

### Run petalTongue:
```bash
cargo run --release -p petal-tongue-ui

# With BiomeOS integration
BIOMEOS_URL=http://localhost:3000 cargo run --release -p petal-tongue-ui
```

### Run BingoCube:
```bash
# Interactive demo
cd bingoCube/demos
cargo run --release

# Run tests
cd bingoCube
cargo test --all
```

### Development:
```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --all

# Check formatting
cargo fmt --all --check

# Build documentation
cargo doc --no-deps --open
```

---

## 📚 **Next Steps**

- **New Users**: Read `README.md` for overview
- **Developers**: See `START_HERE.md` for navigation
- **Status**: Check `STATUS.md` for current metrics
- **Architecture**: Review `specs/` directory

---

## 🗂️ **Project Structure**

```
petalTongue/
├── crates/              # petalTongue source code
├── bingoCube/           # Standalone BingoCube tool
├── showcase/            # Demonstrations
├── specs/               # Technical specifications
└── docs at root         # You are here!
```

---

## ✅ **Verify Installation**

```bash
# Should all pass:
cargo build --all --release   # ✅ ~2.61s
cargo test --all               # ✅ 53 tests
cd bingoCube && cargo test     # ✅ 9 tests
```

---

**Need help?** See `START_HERE.md` for role-based navigation.

