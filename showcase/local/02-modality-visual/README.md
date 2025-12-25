# 02 - Visual 2D Modality Showcase

**Duration**: 15 minutes  
**Purpose**: Demonstrate complete Visual 2D rendering capabilities

---

## 🎨 What This Demo Shows

This demonstrates petalTongue's **Visual 2D Modality** - a complete, production-ready rendering system featuring:

1. **4 Layout Algorithms**: Force-Directed, Hierarchical, Circular, Random
2. **Interactive Controls**: Pan, Zoom, Selection
3. **Health Visualization**: Color-coded states (Green/Yellow/Red/Gray)
4. **Real-Time Updates**: Auto-refresh topology changes
5. **Statistics Overlay**: Node count, edge count, avg degree
6. **Camera Controls**: World ↔ Screen coordinate conversion

---

## 🚀 Quick Start

```bash
./demo.sh
```

This will run through all visual modality features.

---

## 📋 Demo Scenarios

### Scenario 1: Layout Algorithms

**Script**: `./demo-layouts.sh`

**What you'll see**:
1. 5-node topology with **Force-Directed** layout
   - Physics-based, organic positioning
   - Nodes repel, edges attract
   - Natural clustering
2. Switch to **Hierarchical** layout
   - Top-down tree structure
   - Clear parent-child relationships
   - Organized layers
3. Switch to **Circular** layout
   - Nodes arranged in a circle
   - Symmetrical, predictable
   - Good for equal relationships
4. Switch to **Random** layout
   - Arbitrary positioning
   - Useful for testing
   - Comparison baseline

**Key Learning**: Same data, 4 different visual representations!

---

### Scenario 2: Interactive Controls

**Script**: `./demo-interaction.sh`

**What you'll try**:
1. **Pan**: Click and drag to move view
2. **Zoom In**: Mouse wheel up or keyboard `+`
3. **Zoom Out**: Mouse wheel down or keyboard `-`
4. **Select Node**: Click on a node to see details
5. **Reset Camera**: Button to return to default view
6. **Hover**: Mouse over nodes for tooltips

**Key Learning**: Fully interactive, responsive controls!

---

### Scenario 3: Health State Visualization

**Script**: `./demo-health-states.sh`

**What you'll see**:
1. **All Healthy** (Green)
   - System operating normally
   - All nodes green circles
2. **Warning State** (Yellow)
   - One primal reports warning
   - That node turns yellow
   - Others remain green
3. **Critical State** (Red)
   - One primal reports critical
   - That node turns red
   - Visual alert!
4. **Degraded State** (Gray)
   - Primal becomes unreachable
   - Node turns gray
   - Clear visual difference

**Key Learning**: Health states are immediately visible!

---

### Scenario 4: Real-Time Updates

**Script**: `./demo-realtime.sh`

**What you'll see**:
1. Start with 2 primals
2. Add 3rd primal while watching
   - Auto-refresh (5s)
   - New node appears
   - Layout adjusts
3. Stop one primal
   - Node disappears or grays out
   - Layout re-adjusts
4. Add 2 more primals
   - Topology grows
   - All visible immediately

**Key Learning**: Live, real-time topology visualization!

---

### Scenario 5: Scale Testing

**Script**: `./demo-scale.sh`

**What you'll test**:
1. **10 nodes**: Should be smooth, clear
2. **20 nodes**: Should still be performant
3. **50 nodes**: Performance testing
4. **Statistics**: FPS, memory usage

**Key Learning**: Understand performance limits!

---

## ✅ Success Criteria

After this demo, you should understand:

- [x] All 4 layout algorithms and when to use each
- [x] How to interact with the graph (pan, zoom, select)
- [x] How health states are visualized
- [x] How real-time updates work
- [x] Performance characteristics at scale

---

## 🎨 Visual Modality Features

### Complete ✅
- Interactive 2D graph
- 4 layout algorithms
- Health color coding (4 states)
- Pan and zoom
- Node selection
- Camera reset
- Statistics overlay
- World ↔ screen coordinates
- Responsive to window resize

### Accessibility ✅
- Clear visual hierarchy
- Color-blind considerations
- High contrast capable
- Keyboard navigation ready

---

## 📊 What You'll See

### Force-Directed Layout
```
      ●───●
     /│   │\
    ● ●   ● ●
      \   /
       ●─●
```
Organic, physics-based positioning

### Hierarchical Layout
```
       ●
      /│\
     ● ● ●
    /│  \
   ● ●   ●
```
Top-down tree structure

### Circular Layout
```
    ●   ●
   ●     ●
    ●   ●
     ●─●
```
Symmetrical circle arrangement

### Color Coding
- 🟢 **Green**: Healthy (system normal)
- 🟡 **Yellow**: Warning (attention needed)
- 🔴 **Red**: Critical (immediate action)
- ⚫ **Gray**: Degraded (unreachable)

---

## 🌱 Fermentation Notes

### Known Limitations
- Large graphs (100+ nodes) may slow down
- Hierarchical layout requires clear parent-child relationships
- Circular layout best for < 20 nodes

### Potential Improvements
- 3D layout option
- Custom color themes
- Label font size controls
- Edge weight visualization
- Animation transitions between layouts

---

## ⏭️ Next Steps

Once you've explored the visual modality:

```bash
cd ../03-modality-audio/
cat README.md
```

This will show you the **Audio Sonification Modality**!

---

**Status**: 🌱 Ready to build  
**Complexity**: Medium  
**Dependencies**: 01-single-primal complete

---

*"Seeing is one way to understand. But it's just the beginning!"* 🌸

