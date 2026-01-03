# Capability Badges Visualization

**Date**: January 3, 2026  
**Status**: ✅ COMPLETE  
**Component**: `petal-tongue-graph/visual_2d.rs`

---

## 🎯 Overview

Capability badges provide visual indicators of each primal's capabilities directly on the node visualization. They appear as small, color-coded icons orbiting around nodes when sufficiently zoomed in.

---

## 🎨 Visual Design

### Badge Display Rules

- **Zoom Level**: Badges appear at zoom > 0.9 (more detail zoom)
- **Position**: Orbit around the node in a circle
- **Layout**: Up to 6 capability badges displayed
- **Overflow**: "+N" badge shown if more than 6 capabilities

### Badge Appearance

```
Node Structure (zoomed in):
    
         [🔒]              Trust badge (top-right)
           |
    [💾]--●--[⚙️]         Capability badges (orbit)
           |              
         [🔍]             
```

- **Badge Size**: 8.0 * zoom (scaled with zoom level)
- **Orbit Radius**: Node radius + 15.0
- **Background**: Capability color at 30% opacity
- **Border**: 1.5px stroke in capability color
- **Icon**: White emoji/symbol

---

## 🏷️ Capability Icon Mapping

### Security & Trust 🔒
**Color**: Red (255, 100, 100)
- Keywords: `security`, `trust`, `auth`
- Examples: `security.trust`, `authentication`, `authorization`

### Storage 💾
**Color**: Blue (100, 150, 255)
- Keywords: `storage`, `persist`, `data`
- Examples: `permanent_storage`, `data.persist`

### Compute ⚙️
**Color**: Green (150, 200, 100)
- Keywords: `compute`, `container`, `workload`, `execution`
- Examples: `container_runtime`, `workload_execution`

### Discovery 🔍
**Color**: Purple (200, 150, 255)
- Keywords: `discovery`, `orchestr`, `federation`
- Examples: `service_discovery`, `federation.peer`

### Identity 🆔
**Color**: Orange (255, 200, 100)
- Keywords: `identity`, `lineage`, `genetic`
- Examples: `identity.verify`, `genetic.lineage`

### Encryption 🔐
**Color**: Pink (255, 150, 200)
- Keywords: `encrypt`, `crypto`, `sign`
- Examples: `encryption.aes`, `signing.ed25519`

### AI/Inference 🧠
**Color**: Violet (200, 100, 255)
- Keywords: `ai`, `inference`, `intent`, `planning`
- Examples: `intent_parsing`, `task_planning`

### Network 🌐
**Color**: Cyan (100, 200, 255)
- Keywords: `network`, `tcp`, `http`, `grpc`
- Examples: `network.tcp`, `http.server`

### Attribution 📋
**Color**: Peach (255, 200, 150)
- Keywords: `attribution`, `provenance`, `audit`
- Examples: `attribution.track`, `audit.log`

### Visualization 👁️
**Color**: Mint (150, 255, 200)
- Keywords: `visual`, `ui`, `display`
- Examples: `visual.2d`, `ui.interactive`

### Audio 🔊
**Color**: Coral (255, 150, 100)
- Keywords: `audio`, `sound`, `sonification`
- Examples: `audio.output`, `sonification.data`

### Default •
**Color**: Gray
- Used for unknown/unrecognized capabilities

---

## 🎮 User Experience

### Progressive Disclosure

1. **Zoom 0.5+**: Node labels appear
2. **Zoom 0.7+**: Trust badges appear
3. **Zoom 0.9+**: **Capability badges appear** ← This feature

This ensures the UI doesn't become cluttered at lower zoom levels.

### Badge Positioning

Badges are evenly distributed around the node in a circular pattern:

```rust
angle = i * TAU / num_caps  // Evenly spaced
x = center.x + orbit_radius * cos(angle)
y = center.y + orbit_radius * sin(angle)
```

This creates a visually balanced appearance regardless of the number of capabilities.

---

## 📊 Implementation Details

### Code Location

`crates/petal-tongue-graph/src/visual_2d.rs`

### Key Functions

#### `draw_capability_badges()`
```rust
fn draw_capability_badges(
    &self,
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    capabilities: &[String],
)
```

**Purpose**: Draw capability badges orbiting around a node

**Logic**:
1. Limit to 6 displayed capabilities
2. Calculate evenly-spaced positions in orbit
3. Map each capability to icon and color
4. Draw background circle (translucent)
5. Draw border (solid color)
6. Draw icon (white)
7. Show "+N" badge if more capabilities exist

#### `capability_to_icon_and_color()`
```rust
fn capability_to_icon_and_color(capability: &str) -> (&'static str, Color32)
```

**Purpose**: Map capability string to visual representation

**Logic**:
1. Convert capability to lowercase
2. Check for keyword matches (security, storage, compute, etc.)
3. Return appropriate (icon, color) tuple
4. Default to ("•", GRAY) for unknown

---

## 🧪 Testing

### Visual Testing Scenarios

1. **Single Capability**
   - Node: BearDog with `security.trust`
   - Expected: Single 🔒 badge (red)

2. **Multiple Capabilities** 
   - Node: ToadStool with `compute`, `container`, `workload`
   - Expected: 3 badges (⚙️ icons, green)

3. **Maximum Display**
   - Node with 6 capabilities
   - Expected: All 6 badges orbiting node

4. **Overflow**
   - Node with 10 capabilities
   - Expected: 6 badges + "+4" badge

5. **Mixed Categories**
   - Node with security, storage, compute, discovery
   - Expected: 🔒💾⚙️🔍 badges in different colors

### Zoom Level Testing

```bash
# At zoom 0.5: Only node labels
# At zoom 0.7: + Trust badges  
# At zoom 0.9: + Capability badges ✅
```

---

## 🎯 Design Rationale

### Why Badges?

1. **At-a-Glance Understanding**: Instantly see what a primal can do
2. **Visual Categorization**: Color-coding helps identify primal types
3. **Scalable**: Works with 1-12+ capabilities per node
4. **Non-Intrusive**: Only appears at high zoom levels
5. **Accessibility**: Uses both color AND icons

### Why Emoji Icons?

1. **Universal**: No language barrier
2. **Visually Distinct**: Easy to recognize at small sizes
3. **Platform Support**: Works on all modern systems
4. **Accessible**: Screen readers can announce emoji

### Why Orbit Layout?

1. **Symmetrical**: Visually balanced regardless of count
2. **Scalable**: Works with 1-6+ badges
3. **Clear Association**: Obviously belongs to the node
4. **Non-Overlapping**: Badges don't collide with trust badge or family ring

---

## 🔮 Future Enhancements

### Hover Tooltips (Next Session)
- Show full capability name on hover
- Display capability version/details
- Link to capability documentation

### Interactive Badges
- Click badge to filter by capability
- Right-click to see capability details
- Drag to rearrange (custom ordering)

### Advanced Filtering
- "Show only nodes with capability X"
- "Highlight all security capabilities"
- Capability-based graph layout

### Customization
- User-defined icon mappings
- Custom color schemes
- Badge size preferences

---

## 📝 Related Features

- **Trust Visualization** (`trust_level_to_colors()`) - Completed
- **Family ID Rings** (`family_id_to_color()`) - Completed  
- **Health Status Colors** (`health_to_colors()`) - Existing
- **Hover Tooltips** - Next session

---

## 🎊 Success Metrics

✅ **Implemented**: Capability badges with 11+ icon types  
✅ **Tested**: Builds successfully, zero errors  
✅ **Deployed**: Binary updated in `../primalBins/`  
✅ **Documented**: This comprehensive guide  
✅ **Scalable**: Handles 1-12+ capabilities gracefully  
✅ **Accessible**: Color + icon dual encoding

---

**Status**: ✅ COMPLETE - Ready for integration testing  
**Next**: Hover tooltips for full capability details

🎨 **Visual richness achieved - Capabilities now visible at a glance!** 🚀

