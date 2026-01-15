# 🎮 Testing Doom in petalTongue

**Status**: Ready to test!  
**What Works**: First-person view, movement, map rendering  
**What's Needed**: A Doom WAD file

---

## 🚀 **Quick Start**

### **Step 1: Get a WAD File**

You need a Doom WAD file. Two options:

#### **Option A: Freedoom** (100% Free, Open Content)
```bash
# Ubuntu/Debian
sudo apt install freedoom

# The WAD will be at:
# /usr/share/games/doom/freedoom1.wad

# Copy to project root:
cp /usr/share/games/doom/freedoom1.wad .
```

#### **Option B: Doom Shareware** (Free, Original Assets)
```bash
# Download shareware Doom
wget https://distro.ibiblio.org/slitaz/sources/packages/d/doom1.wad

# Or from archive.org:
wget https://archive.org/download/DoomsharewareEpisode/doom.ZIP
unzip doom.ZIP
# Extract DOOM1.WAD and rename to lowercase:
mv DOOM1.WAD doom1.wad
```

### **Step 2: Run petalTongue**

```bash
# Make sure WAD is in project root
ls -lh doom1.wad  # or freedoom1.wad

# Build and run
cargo run --release --bin petal-tongue -- \
  --scenario sandbox/scenarios/doom-mvp.json
```

### **Step 3: Explore!**

You should see:
- E1M1 rendered from first-person view
- Sky gradient at top, floor at bottom
- Gray walls in perspective
- Distance-based fog effect

---

## 🎮 **Controls**

### **Movement**
- **W** or **Up Arrow**: Move forward
- **S** or **Down Arrow**: Move backward
- **A**: Strafe left
- **D**: Strafe right

### **Turning**
- **Left Arrow**: Turn left
- **Right Arrow**: Turn right
- **Mouse Movement**: Turn (left/right)

### **View Modes**
- **Tab** (or configure in panel): Toggle 2D ⇄ 3D view

---

## 🔍 **What You'll See**

### **Current Implementation** (Phase 1.2)

✅ **Working**:
- First-person 3D perspective
- All walls rendered correctly
- Proper depth/distance perception
- Distance-based shading (fog)
- Player movement (forward/back/strafe)
- Camera rotation (mouse/keyboard)
- Sky and floor rendering

⏳ **Not Yet Implemented**:
- Wall textures (Phase 1.3 enhancement)
- Enemies (Phase 1.3)
- Weapons (Phase 1.3)
- Item pickups (Phase 1.3)
- Collision detection (Phase 1.3 enhancement)
- Audio (Phase 1.3 enhancement)

### **What to Expect**

You'll see a **simplified Doom** - think early prototype or wireframe mode:
- Walls are solid gray (no textures yet)
- No enemies or items visible
- Can walk through walls (no collision yet)
- Silent (no audio yet)

**But the core is there!** You're walking through a real Doom map in first-person 3D! 🎯

---

## 🧪 **Testing Checklist**

### **Basic Functionality**
- [ ] WAD file loads successfully
- [ ] Map E1M1 is displayed
- [ ] First-person view is visible
- [ ] Walls appear in correct perspective
- [ ] Sky is visible at top
- [ ] Floor is visible at bottom

### **Movement**
- [ ] W key moves forward
- [ ] S key moves backward
- [ ] A key strafes left
- [ ] D key strafes right
- [ ] Arrow keys turn left/right
- [ ] Mouse turning works

### **Visual Quality**
- [ ] Walls have proper height
- [ ] Distance fog effect visible
- [ ] Perspective looks correct
- [ ] No visual glitches
- [ ] Smooth rendering (60 FPS)

---

## 🐛 **Troubleshooting**

### **"No WAD file found"**

Error: `WadNotFound("No WAD file found...")`

**Solution**: 
- Ensure WAD file is in project root
- Check filename: `doom1.wad` or `freedoom1.wad` (lowercase)
- Try absolute path: `DOOM_WAD=/path/to/doom1.wad cargo run...`

### **"Failed to load WAD"**

Error: `InvalidWad(...)`

**Solution**:
- WAD file might be corrupted
- Re-download from trusted source
- Check file size (doom1.wad should be ~2.4 MB)

### **Black screen / No rendering**

**Solution**:
- Check terminal output for errors
- Ensure graphics drivers are working
- Try different WAD file
- Check if map has data: "Parsed map E1M1 with X vertices..."

### **Poor performance**

**Solution**:
- Use release build: `cargo run --release ...`
- Reduce resolution (edit scenario JSON)
- Close other applications
- Update graphics drivers

---

## 📊 **Expected Performance**

### **Target**
- **FPS**: 60+ (smooth)
- **Frame time**: <16ms
- **CPU**: <25% on modern hardware

### **Actual** (will vary)
- Desktop: Should hit 60 FPS easily
- Laptop: 30-60 FPS depending on specs
- Low-end: May need resolution reduction

---

## 🎯 **Advanced Testing**

### **Different Maps**

The WAD contains multiple maps. To test others:

```rust
// In doom_instance.rs, change:
self.load_map("E1M2")?;  // Instead of E1M1
```

Maps in doom1.wad: E1M1 through E1M9

### **Performance Profiling**

```bash
# With perf (Linux)
perf record cargo run --release --bin petal-tongue -- \
  --scenario sandbox/scenarios/doom-mvp.json

perf report

# With flamegraph
cargo install flamegraph
cargo flamegraph --bin petal-tongue -- \
  --scenario sandbox/scenarios/doom-mvp.json
```

### **Memory Usage**

```bash
# Monitor memory
watch -n 1 'ps aux | grep petal-tongue'

# Expected:
# ~50-100 MB for WAD data
# ~10-20 MB for rendering
# Total: ~60-120 MB
```

---

## 🌸 **What's Next**

After testing Phase 1.2, you can:

1. **Phase 1.3**: Add gameplay
   - Enemy sprites
   - Weapons & shooting
   - Item pickups

2. **Phase 1.4**: Add live stats
   - biomeOS proprioception overlay
   - neuralAPI metrics while playing
   - System health monitoring

3. **Enhancements**:
   - BSP tree optimization
   - Wall texture mapping
   - Collision detection
   - Audio support

4. **Phase 2**: petalTongue IS Doom!
   - Your biome becomes the game world
   - Primals become entities
   - System admin becomes gameplay

---

## 💡 **Tips**

### **For Best Experience**
- Use a mouse for smooth turning
- Start in a corner to get bearings
- Look for distinctive room shapes
- Remember E1M1 layout (hangar bay)

### **If Something's Wrong**
- Check terminal output for errors
- Look for "Parsed map E1M1 with X vertices"
- Verify "Player start: (X, Y) angle: Z"
- Watch for panic messages

### **For Development**
- Enable debug logs: `RUST_LOG=debug cargo run...`
- Check rendering stats in terminal
- Profile with release build
- Test with different WAD files

---

## 📚 **Resources**

### **WAD Files**
- Freedoom: https://freedoom.github.io/
- Doom shareware: https://doomwiki.org/wiki/DOOM1.WAD
- Archive.org: https://archive.org/details/DoomsharewareEpisode

### **Documentation**
- `DOOM_PHASE_1_2_PLAN.md` - Implementation details
- `PETALTONGUE_IS_DOOM_VISION.md` - Full vision
- `EXTRAORDINARY_SESSION_JAN_15_2026_FINAL.md` - Session summary

### **Code**
- `crates/doom-core/src/wad_loader.rs` - WAD parsing
- `crates/doom-core/src/raycast_renderer.rs` - First-person rendering
- `crates/doom-core/src/lib.rs` - DoomInstance

---

## ✅ **Success Criteria**

You've successfully tested Phase 1.2 if you can:
- [x] Load a WAD file
- [x] See E1M1 from first-person view
- [x] Walk around the map
- [x] Turn with mouse/keyboard
- [x] Recognize the Doom hangar bay layout

---

**Status**: Ready to test!  
**Difficulty**: Easy (just need a WAD file)  
**Time**: 5 minutes to test  
**Fun**: HIGH! 🎮  

🌸 **Enjoy walking through Doom in petalTongue!** 🌸

