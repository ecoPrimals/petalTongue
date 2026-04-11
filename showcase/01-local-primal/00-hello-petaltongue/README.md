# 🌸 Hello petalTongue

**Your First Visualization**

---

## 🎯 What You'll Learn

In just **5 minutes**, you'll:
- Launch petalTongue for the first time
- See your first graph visualization
- Hear your first audio sonification
- Understand multi-modal representation

**No BiomeOS required.** Just petalTongue.

---

## ⏱️ Duration

**5 minutes** (quick introduction)

---

## 📋 Prerequisites

- Rust installed (`rustc --version`)
- petalTongue built (`cargo build --release`)

### Build if needed:

```bash
cd /path/to/petalTongue
cargo build --release
```

---

## 🚀 Run the Demo

```bash
./demo.sh
```

Or manually:

```bash
# Launch petalTongue with a simple mock graph
cargo run --release
```

---

## 👀 What You'll See

### **Visual Representation**

The petalTongue window will open showing:
- **3 nodes** in a circular layout:
  - 🟦 Blue node (Discovery primal)
  - 🟩 Green node (Security primal)
  - 🟧 Orange node (Storage primal)
- **2 edges** connecting them
- **Interactive controls** (zoom, pan, layout selection)

### **Audio Representation**

You'll hear (if audio is enabled):
- **3 distinct tones**:
  - 🎵 Light chimes (Discovery)
  - 🎵 Deep bass (Security)
  - 🎵 Strings (Storage)
- **Spatial positioning** (left/center/right)
- **Harmonic tones** (all nodes healthy)

### **What This Proves**

✅ **Same information, different channels**  
✅ **Visual users see the graph**  
✅ **Blind users hear the graph**  
✅ **Both get complete information**

---

## 🎓 Key Concepts

### **1. Multi-Modal Rendering**

The **same data** (graph with 3 nodes) is rendered in:
- **Visual modality** → Shapes, colors, positions
- **Audio modality** → Instruments, pitch, spatial

Both represent the complete information.

### **2. Graph Engine Architecture**

```
GraphEngine (Modality-Agnostic)
         ↓
    ┌────┴────┐
    ↓         ↓
Visual 2D   Audio
Renderer   Renderer
```

The engine has **no knowledge** of rendering. Renderers consume its data.

### **3. Health State Mapping**

| Health State | Visual | Audio |
|--------------|--------|-------|
| Healthy | Green | Harmonic (in-key) |
| Warning | Yellow | Off-key |
| Critical | Red | Dissonant |

### **4. Node Type Mapping**

| Primal Type | Visual | Audio |
|-------------|--------|-------|
| Discovery (Songbird) | Blue | Chimes |
| Security (BearDog) | Green | Bass |
| Storage (NestGate) | Orange | Strings |
| Compute (ToadStool) | Purple | Drums |
| AI (Squirrel) | Pink | Synth |

---

## 💡 Try This

### **Change the Layout**

Click the layout dropdown:
- **Force-Directed** - Nodes push/pull naturally
- **Circular** - Nodes in a circle
- **Hierarchical** - Tree structure
- **Random** - Random positions

### **Zoom and Pan**

- **Mouse wheel** - Zoom in/out
- **Click + drag** - Pan around
- **Click node** - Select it

### **Toggle Audio**

- Check "Enable Audio" to hear the soundscape
- Uncheck to mute
- Notice how **same info** is presented differently

---

## 📊 What This Demonstrates

1. ✅ **petalTongue works** - Basic functionality validated
2. ✅ **Multi-modal rendering** - Visual + Audio both operational
3. ✅ **Interactive UI** - Zoom, pan, layout selection
4. ✅ **Accessibility** - Blind users get complete information
5. ✅ **Graph engine** - Modality-agnostic core proven

---

## 🐛 Troubleshooting

### **Window doesn't open**

```bash
# Check if built
ls -la ../../target/release/petal*

# Rebuild if needed
cargo build --release
```

### **No audio**

Audio is optional. Check:
- Is "Enable Audio" checked?
- Do you have audio output device?
- Is volume up?

**Note**: Audio is a **nice-to-have**, not required. petalTongue works perfectly without it.

### **Compilation errors**

```bash
# Clean and rebuild
cargo clean
cargo build --release
```

---

## 🎯 Success Criteria

You've succeeded when you:
- ✅ See the petalTongue window
- ✅ See 3 nodes and 2 edges
- ✅ Can zoom and pan
- ✅ Can switch layouts
- ✅ (Optional) Hear the audio

---

## ➡️ Next Steps

**You're ready for the next demo!**

```bash
cd ../01-graph-engine/
cat README.md
```

**Next demo**: Learn about the graph engine and its 4 layout algorithms.

---

## 📚 Learn More

- [Main README](../../../README.md) - Project overview
- [Specs](../../../specs/) - Architecture specifications
- [Getting Started](../../../START_HERE.md) - Development setup

---

## 🌟 What You've Accomplished

**Congratulations!** 🎉

In just 5 minutes, you've:
- ✅ Launched petalTongue
- ✅ Seen multi-modal rendering
- ✅ Understood core concepts
- ✅ Validated accessibility

**You're now ready to explore petalTongue's full capabilities!**

---

*"Every expert was once a beginner. Welcome to petalTongue!"* 🌸

