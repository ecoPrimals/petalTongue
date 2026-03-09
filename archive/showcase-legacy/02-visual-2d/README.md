# 🎨 Visual 2D - Interactive Visualization

**Master interactive graph visualization and controls**

---

## 🎯 What You'll Learn

In **10 minutes**:
- Interactive zoom, pan, and selection
- Visual encoding (colors, shapes, sizes)
- Node and edge rendering
- Control panel usage

---

## ⏱️ Duration

**10 minutes**

---

## 📋 Prerequisites

- petalTongue built
- Completed: 00-hello, 01-graph-engine

---

## 🚀 Run the Demo

```bash
./demo.sh
```

---

## 🎓 Visual Encoding System

### **Node Colors = Primal Type**

| Primal | Color | Hex | Meaning |
|--------|-------|-----|---------|
| Songbird (Discovery) | 🟦 Blue | #4A90E2 | Finding services |
| BearDog (Security) | 🟩 Green | #50C878 | Authentication |
| NestGate (Storage) | 🟧 Orange | #FF8C42 | Data persistence |
| ToadStool (Compute) | 🟣 Purple | #9B59B6 | Task execution |
| Squirrel (AI) | 🟪 Pink | #E91E63 | Intelligence |

### **Node Shape = Health State**

| Health | Visual | Border |
|--------|--------|--------|
| Healthy | Solid circle | Thin green |
| Warning | Pulsing | Thick yellow |
| Critical | Flashing | Thick red |
| Unknown | Dashed | Gray |

### **Edge Thickness = Bandwidth**

- Thin line: Low traffic (< 1 MB/s)
- Medium line: Moderate (1-10 MB/s)
- Thick line: High traffic (> 10 MB/s)

---

## 👀 What You'll See

### **Interactive Controls**

**Mouse Controls**:
- **Scroll Wheel**: Zoom in/out
- **Left Click + Drag**: Pan around
- **Left Click Node**: Select node
- **Right Click**: Context menu (future)

**Keyboard Controls**:
- **Arrow Keys**: Pan direction
- **+/-**: Zoom in/out
- **Space**: Reset view
- **Escape**: Deselect

### **Control Panel**

```
┌─ Controls ──────────────────┐
│ Layout: [Force-Directed ▼] │
│ □ Show Labels              │
│ □ Show Health Indicators   │
│ □ Enable Animation         │
│ Zoom: 100% [━━●━━]         │
└─────────────────────────────┘
```

### **Selection Details**

When you select a node:
```
┌─ Selected: songbird-1 ──────┐
│ Type: Discovery (Songbird)  │
│ Health: Healthy ✓           │
│ Connections: 3 edges        │
│ CPU: 23% | Memory: 156 MB   │
│ Uptime: 2h 34m              │
└──────────────────────────────┘
```

---

## 💡 Interactive Features

### **1. Zoom Levels**

Try different zoom levels:
- **25%** - Overview of entire graph
- **100%** - Default view  
- **200%** - Detailed inspection
- **400%** - Maximum detail

### **2. Pan Navigation**

Navigate large graphs:
- Drag to move viewport
- Arrow keys for precise movement
- Minimap shows current view (if > 10 nodes)

### **3. Node Selection**

Click nodes to:
- View detailed information
- Highlight connections
- See real-time metrics
- Access actions (future)

### **4. Visual Feedback**

Watch for:
- **Hover**: Node highlights
- **Selection**: Thick border
- **Connection**: Related edges highlight
- **Health**: Visual indicators pulse

---

## 📊 What This Demonstrates

1. ✅ **Rich Visual Encoding** - Color, shape, size convey info
2. ✅ **Interactive Exploration** - Zoom, pan, select
3. ✅ **Real-Time Updates** - Dynamic visualization
4. ✅ **Intuitive Controls** - Mouse + keyboard
5. ✅ **Accessibility** - Keyboard navigation works

---

## 🎨 Visual Design Principles

### **Clarity**
- High contrast colors
- Clear boundaries
- Readable text
- Distinct shapes

### **Information Density**
- Show important info prominently
- Hide details until needed
- Progressive disclosure
- Clean layout

### **Interactivity**
- Immediate feedback
- Smooth transitions
- Predictable behavior
- Undo/redo support (future)

---

## 🐛 Troubleshooting

### **Window too small**
Resize the window or use zoom controls.

### **Can't see all nodes**
- Zoom out (scroll or -)
- Use pan (drag or arrows)
- Try different layout

### **Performance issues with many nodes**
- Disable animations
- Use simpler layout (Circular/Random)
- Check system resources

---

## 🎯 Success Criteria

You've mastered visual 2D when you:
- ✅ Can zoom and pan smoothly
- ✅ Understand color encoding
- ✅ Can select and inspect nodes
- ✅ Use keyboard shortcuts
- ✅ Navigate large graphs

---

## ➡️ Next Steps

```bash
cd ../03-audio-sonification/
cat README.md
```

**Next**: Learn audio sonification and spatial sound.

---

## 📚 Technical Details

### **Rendering Stack**

```
Visual2DRenderer
├── egui (Immediate mode GUI)
├── Painter (2D drawing)
└── Transform (Screen ↔ World coords)
```

### **Performance**

- **10 nodes**: 60 FPS
- **50 nodes**: 60 FPS
- **100 nodes**: 45-60 FPS
- **500 nodes**: 30-45 FPS

Rendering is GPU-accelerated where available.

### **Coordinate Systems**

```rust
// World coordinates (graph space)
let world_pos = Vec2::new(100.0, 200.0);

// Screen coordinates (pixel space)
let screen_pos = renderer.world_to_screen(world_pos);

// Mouse to world
let world_click = renderer.screen_to_world(mouse_pos);
```

---

## 🌟 Key Takeaway

**Visual 2D rendering is one modality.**

The **same graph** can also be:
- 🎵 Sonified to audio (next demo)
- 📊 Exported to data
- 📱 Rendered on mobile
- 🥽 Visualized in VR

**Modality-agnostic design enables all of this!**

---

*"Great visualization makes complexity understandable."* 🌸

