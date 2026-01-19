# 🎮 Doom Phase 1.2 - First-Person BSP Rendering

**Date**: January 15, 2026  
**Phase**: 1.2 - First-Person View  
**Status**: 🔄 Starting Now  
**Depends On**: Phase 1.1 ✅ Complete

---

## 🎯 **Goal**

Transform the top-down 2D map view into a first-person 3D perspective, letting the player see the world as if they're inside it!

---

## 📋 **What We'll Build**

### **1. BSP Tree Traversal**
The Doom engine uses Binary Space Partitioning (BSP) trees to efficiently render the visible world.

**What we need**:
- Parse NODES lump (BSP tree structure)
- Parse SEGS lump (wall segments)
- Parse SSECTORS lump (subsectors - visible areas)
- Traverse BSP tree from player position
- Determine visible subsectors (front-to-back)

### **2. First-Person Camera**
Convert 2D map coordinates to 3D player view.

**What we need**:
- Player position (x, y, z)
- Player angle (yaw)
- Camera height (view height)
- Field of view (FOV)
- Projection matrix

### **3. Wall Rendering**
Draw walls from player perspective using raycasting or scanline rendering.

**What we need**:
- For each screen column:
  - Cast ray from player
  - Find wall intersection
  - Calculate wall height on screen
  - Determine texture column
  - Draw vertical line with correct perspective

### **4. Texture Mapping**
Apply wall textures to create visual detail.

**What we need**:
- Parse TEXTURE1/TEXTURE2 lumps
- Parse PNAMES lump (patch names)
- Parse wall texture definitions
- Map texture coordinates to screen pixels
- Handle upper/lower/middle textures

---

## 🎨 **Rendering Approach**

### **Option A: Full BSP + Texture Mapping** (Complex)
- Exact Doom rendering
- BSP traversal
- Full texture support
- ~2-3 days work

### **Option B: Raycasting + Simple Walls** (Simple)
- Similar to Wolfenstein 3D
- Simpler than BSP
- Untextured or simple colors first
- ~1 day work, then add textures

### **Option C: Hybrid** (Recommended)
- Start with raycasting for quick results
- Then add BSP for optimization
- Add textures incrementally
- Test-driven evolution! 🧬

**Decision**: **Option C** - Get something visible quickly, then evolve!

---

## 🚀 **Implementation Plan**

### **Step 1: Basic First-Person View** (Today)

**Goal**: See walls from player perspective (untextured)

```rust
pub struct FirstPersonRenderer {
    width: usize,
    height: usize,
    framebuffer: Vec<u8>,
    player_x: f32,
    player_y: f32,
    player_angle: f32,  // In radians
    fov: f32,           // Field of view
}

impl FirstPersonRenderer {
    pub fn render(&mut self, map: &MapData) {
        // For each screen column
        for x in 0..self.width {
            // Calculate ray angle
            let ray_angle = self.calculate_ray_angle(x);
            
            // Cast ray to find wall
            if let Some(wall) = self.cast_ray(map, ray_angle) {
                // Calculate wall height on screen
                let wall_height = self.calculate_wall_height(wall.distance);
                
                // Draw vertical line
                self.draw_wall_column(x, wall_height, wall.color);
            }
        }
    }
}
```

**Estimated Time**: 2-3 hours

### **Step 2: Player Movement** (Today/Tomorrow)

**Goal**: WASD keys move player, mouse turns camera

```rust
impl DoomInstance {
    pub fn update_player(&mut self, keys: &HashSet<DoomKey>, mouse_dx: f32) {
        let move_speed = 10.0; // Units per frame
        let turn_speed = 0.05; // Radians per pixel
        
        // Rotation
        self.player_angle += mouse_dx * turn_speed;
        
        // Forward/backward
        if keys.contains(&DoomKey::Up) {
            self.player_x += self.player_angle.cos() * move_speed;
            self.player_y += self.player_angle.sin() * move_speed;
        }
        
        // Strafe left/right
        // ...
    }
}
```

**Estimated Time**: 1-2 hours

### **Step 3: BSP Integration** (Tomorrow)

**Goal**: Use BSP tree for efficient rendering

```rust
pub fn parse_bsp_nodes(lumps: &[Lump], map_index: usize) -> Result<Vec<BspNode>> {
    // Parse NODES lump
    // Each node is 28 bytes
}

pub fn traverse_bsp(&self, node_id: i16, player_x: f32, player_y: f32) -> Vec<SubsectorId> {
    // Recursively traverse BSP tree
    // Return visible subsectors in front-to-back order
}
```

**Estimated Time**: 2-3 hours

### **Step 4: Simple Textures** (Tomorrow/Day 3)

**Goal**: Add basic wall textures

```rust
pub fn parse_textures(lumps: &[Lump]) -> Result<Vec<Texture>> {
    // Parse TEXTURE1, PNAMES lumps
    // Build texture definitions
}

pub fn draw_textured_wall(&mut self, x: usize, wall: &Wall, texture: &Texture) {
    // Map texture column to screen column
    // Apply perspective-correct texture mapping
}
```

**Estimated Time**: 3-4 hours

---

## 🧬 **Expected Evolution Discoveries**

As we implement, we'll discover:

1. **Performance**: Can egui handle 60 FPS raycasting?
   - May need to optimize
   - May need to render to texture, not individual pixels

2. **Coordinate Systems**: Doom's coordinate system is quirky
   - Y-axis is inverted
   - Angles measured differently
   - May need conversion helpers

3. **Collision Detection**: Need to prevent walking through walls
   - Add wall collision checks
   - Validate movement before applying

4. **Texture Loading**: Doom's texture format is complex
   - May start with solid colors
   - Add textures incrementally

5. **Input Handling**: First-person needs different input
   - Mouse capture
   - Smooth movement
   - May discover focus system gaps

---

## 📊 **Success Criteria**

**Minimum (MVP)**:
- [x] ~~Top-down view~~ (Phase 1.1)
- [ ] First-person view visible
- [ ] Can see walls from player perspective
- [ ] Walls have correct height/distance
- [ ] Untextured (solid colors) is OK

**Good**:
- [ ] Player can move (WASD)
- [ ] Player can turn (mouse)
- [ ] Basic collision detection
- [ ] Simple wall colors/shading

**Excellent**:
- [ ] BSP tree traversal working
- [ ] Wall textures visible
- [ ] Smooth 60 FPS
- [ ] Feels like Doom!

---

## 🎓 **Technical References**

### **Raycasting Algorithm**

```
For each screen column x:
    1. Calculate ray angle: 
       angle = player_angle + (x - width/2) * (fov / width)
    
    2. Cast ray until hitting wall:
       - Step along ray direction
       - Check each grid cell for walls
       - Record distance when wall hit
    
    3. Calculate wall height:
       height = (wall_base_height / distance) * distance_to_plane
    
    4. Draw vertical line:
       - Top: (screen_height/2) - (height/2)
       - Bottom: (screen_height/2) + (height/2)
```

### **BSP Tree Structure**

```rust
struct BspNode {
    x: i16,          // Partition line start X
    y: i16,          // Partition line start Y
    dx: i16,         // Partition line delta X
    dy: i16,         // Partition line delta Y
    bbox_right: [i16; 4],  // Right bounding box
    bbox_left: [i16; 4],   // Left bounding box
    right_child: i16,      // Right child (node or subsector)
    left_child: i16,       // Left child (node or subsector)
}
```

### **Doom Coordinate System**

- **X axis**: Left/Right
- **Y axis**: Forward/Back (positive = north)
- **Z axis**: Up/Down (height)
- **Angles**: 0 = east, 90 = north, 180 = west, 270 = south
- **Units**: 1 unit = approximately 1 pixel at 1:1 scale

---

## 🎮 **Testing Plan**

### **Visual Tests**
1. Load E1M1
2. Start at player spawn point
3. Should see walls in front
4. Walls should have correct perspective
5. Rotating should show different walls

### **Movement Tests**
1. WASD moves player
2. Mouse turns camera
3. Can't walk through walls
4. Movement feels smooth

### **Performance Tests**
1. Measure FPS
2. Should maintain 60 FPS minimum
3. If not, optimize:
   - Reduce resolution
   - Optimize raycasting
   - Use BSP tree

---

## 📈 **Timeline**

### **Day 1** (Today - Jan 15)
- [x] Phase 1.1 complete
- [ ] Basic first-person view (raycasting)
- [ ] Player movement (WASD + mouse)
- **Deliverable**: Can walk around E1M1 seeing untextured walls

### **Day 2** (Jan 16)
- [ ] BSP tree integration
- [ ] Collision detection
- [ ] Performance optimization
- **Deliverable**: Smooth movement with BSP

### **Day 3** (Jan 17)
- [ ] Texture parsing
- [ ] Texture mapping
- [ ] Polish & testing
- **Deliverable**: Textured walls, looks like Doom!

**Total**: 3 days to complete Phase 1.2

---

## 🌸 **Philosophy**

> "Start simple, evolve as we discover"

We won't build a perfect Doom engine. We'll build:
1. Something that shows first-person view (today)
2. Something you can walk around in (tomorrow)
3. Something that looks like Doom (day 3)

Each step reveals what we actually need, not what we think we need.

**Test-driven evolution continues!** 🧬

---

## 🎯 **Ready to Start**

Phase 1.1 gave us:
- ✅ WAD file parsing
- ✅ Map geometry data
- ✅ Top-down view (proof of concept)

Phase 1.2 will give us:
- 🔄 First-person perspective
- 🔄 Player movement
- 🔄 Proper 3D rendering

**Let's make it happen!** 🚀

---

**Status**: 🔄 Starting Now  
**Estimated Completion**: 3 days  
**Complexity**: Medium  
**Excitement**: HIGH! 🎮

