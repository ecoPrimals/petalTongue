# petalTongue Quick Start Guide

**Get up and running in 60 seconds!**

---

## 🚀 Fastest Way: Headless Mode

Works everywhere - no GUI required!

```bash
# Clone and build
cd /path/to/petalTongue
cargo build --release -p petal-tongue-headless

# Run terminal UI
./target/release/petal-tongue-headless --mode terminal

# Or export to SVG
./target/release/petal-tongue-headless --mode svg --output topology.svg
```

**That's it!** You're visualizing topology.

---

## 🖥️ Full GUI Mode

For desktop environments with display:

```bash
# Build the full UI
cargo build --release --bin petal-tongue

# Run with tutorial mode
SHOWCASE_MODE=true ./target/release/petal-tongue
```

**Features:** Interactive graph, real-time updates, full visualization

---

## 📊 Export Modes

### Terminal (ASCII Art)

```bash
./target/release/petal-tongue-headless --mode terminal
```

**Use:** SSH sessions, monitoring, tmux

### SVG (Browser-Friendly)

```bash
./target/release/petal-tongue-headless --mode svg --output topology.svg
open topology.svg  # macOS
xdg-open topology.svg  # Linux
```

**Use:** Documentation, reports, web pages

### JSON (API-Friendly)

```bash
./target/release/petal-tongue-headless --mode json --output topology.json
cat topology.json | jq '.topology.primals'
```

**Use:** Data analysis, APIs, automation

### DOT (Graphviz)

```bash
./target/release/petal-tongue-headless --mode dot --output topology.dot
dot -Tpng topology.dot -o diagram.png
```

**Use:** Advanced layouts, publications

---

## 🔧 Environment Variables

```bash
# Enable tutorial mode
export SHOWCASE_MODE=true

# Set biomeOS endpoint (for production)
export BIOMEOS_URL=http://localhost:3000

# Enable debug logging
export RUST_LOG=debug
```

---

## 📦 Use Cases

### Server Monitoring

```bash
# Over SSH
ssh server petal-tongue-headless --mode terminal

# Watch mode
watch -n 5 ssh server petal-tongue-headless
```

### CI/CD Integration

```yaml
# GitHub Actions
- run: cargo build --release -p petal-tongue-headless
- run: ./target/release/petal-tongue-headless --mode svg --output docs/topology.svg
```

### Docker Container

```bash
docker run -it petaltongue-headless --mode terminal
```

---

## 🆘 Troubleshooting

**Binary not found?**
```bash
cargo build --release -p petal-tongue-headless
export PATH=$PATH:$(pwd)/target/release
```

**Need help?**
```bash
petal-tongue-headless --help
```

**Want more details?**
- See [docs/features/HEADLESS_MODE.md](docs/features/HEADLESS_MODE.md)
- See [START_HERE.md](START_HERE.md)

---

## 🌸 You're Ready!

petalTongue is now visualizing your ecoPrimals topology.

**Next steps:**
- Connect to real primals via biomeOS
- Export visualizations for documentation
- Integrate with your monitoring stack

**Questions?** See [DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md)

