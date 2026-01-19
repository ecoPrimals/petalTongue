# 🎨 Interactive Testing Guide - petalTongue v2.3.0

**Date**: January 15, 2026  
**Version**: v2.3.0 (commit e97704b)  
**Status**: ✅ All features deployed and ready to test

---

## 🎯 What You're Testing

This guide will help you verify all the new interactive features that were just deployed:

1. ✅ **Interactive Node Creation** - Double-click to create
2. ✅ **Interactive Edge Creation** - Drag to connect (with validation!)
3. ✅ **Node Deletion** - Select and delete
4. ✅ **Modular UI** - Different scenarios show different panels
5. ✅ **Capability Validation** - Watch console for validation messages

---

## 🚀 Quick Start

### **Start the Application**:
```bash
cd /path/to/petalTongue

# Option 1: Minimal Paint Mode (recommended for first test)
cargo run --release --bin petal-tongue -- \
  --scenario sandbox/scenarios/paint-simple.json

# Option 2: Full Dashboard
cargo run --release --bin petal-tongue -- \
  --scenario sandbox/scenarios/full-dashboard.json

# Option 3: Neural API Focus
cargo run --release --bin petal-tongue -- \
  --scenario sandbox/scenarios/neural-api-test.json
```

### **Enable Logging** (Optional):
```bash
# See capability validation messages
RUST_LOG=petal_tongue_graph=info cargo run --release --bin petal-tongue -- \
  --scenario sandbox/scenarios/paint-simple.json
```

---

## 🧪 Test Scenario 1: Paint Mode (Minimal UI)

**File**: `sandbox/scenarios/paint-simple.json`  
**Purpose**: Clean canvas with no distractions  
**Initial Nodes**: 3 (Alpha, Beta, Gamma)

### **Expected UI**:
```
╔═══════════════════════════════════════════════════════════════╗
║  [Top Menu Bar]                                               ║
╠═══════════════════════════════════════════════════════════════╣
║                                                               ║
║               [Full-Width Canvas]                             ║
║                                                               ║
║    Alpha ●           Beta ●          Gamma ●                  ║
║                                                               ║
║                                                               ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
```

**No Sidebars**: ✅ Confirmed  
**No Graph Statistics**: ✅ Confirmed  
**No System Dashboard**: ✅ Confirmed

### **Test Steps**:

#### **Test 1.1: Create Node via Double-Click**
1. **Action**: Double-click on empty space in the canvas
2. **Expected**: 
   - New node appears at cursor position
   - Node labeled "Node-4" (or similar auto-generated ID)
   - Node has default gray color
3. **Verify**: 
   - Node is visible
   - Node has unique ID
   - Node can be selected (click on it)

#### **Test 1.2: Create Edge via Drag**
1. **Action**: 
   - Click and hold on "Alpha" node
   - Drag cursor toward "Beta" node
   - Release over "Beta"
2. **Expected**:
   - Blue line follows cursor while dragging
   - Line snaps to "Beta" when over it
   - Gray connection line appears between Alpha ↔ Beta
3. **Verify**:
   - Edge is visible
   - Edge connects correct nodes
   - No duplicate edges if you repeat

#### **Test 1.3: Capability Validation (Console)**
1. **Action**: Create edge between two nodes
2. **Expected** (in terminal):
   ```
   ⚠️ Connection warning: No explicit capability match between 'Alpha' and 'Beta'
   ```
   OR
   ```
   ✅ Connection validated
   ```
3. **Verify**: Check terminal for validation messages

#### **Test 1.4: Delete Node**
1. **Action**:
   - Click on "Gamma" node to select it
   - Press **Delete** key (or **Backspace**)
2. **Expected**:
   - "Gamma" node disappears
   - Any connected edges also disappear
3. **Verify**:
   - Node is gone
   - Canvas updates correctly
   - No crash or errors

#### **Test 1.5: Navigation**
1. **Pan**: Click and drag on empty space → Canvas moves
2. **Zoom**: Scroll wheel → Canvas zooms in/out
3. **Verify**: Smooth navigation, nodes move correctly

---

## 🧪 Test Scenario 2: Full Dashboard

**File**: `sandbox/scenarios/full-dashboard.json`  
**Purpose**: All features enabled  
**Initial Nodes**: 5 (NUCLEUS, BearDog, Songbird, Toadstool, User)

### **Expected UI**:
```
╔════════════╦═══════════════════════════════════╦════════════╗
║  [Left     ║  [Top Menu Bar]                   ║  [Right    ║
║  Sidebar]  ╠═══════════════════════════════════╣  Sidebar]  ║
║            ║                                   ║            ║
║  • Nodes   ║        [Main Canvas]              ║  • Metrics ║
║  • Edges   ║                                   ║  • Health  ║
║            ║    NUCLEUS ●                      ║  • Stats   ║
║            ║      ↓                            ║            ║
║            ║    BearDog ●  Songbird ●          ║            ║
║            ║                                   ║            ║
╠════════════╩═══════════════════════════════════╩════════════╣
║  [Bottom: Graph Statistics Window]                          ║
║  Nodes: 5 | Edges: 4 | Avg Degree: 1.6                      ║
╚═════════════════════════════════════════════════════════════╝
```

### **Test Steps**:

#### **Test 2.1: Verify All Panels Visible**
- [x] Left Sidebar showing
- [x] Right Sidebar showing
- [x] Graph Statistics panel (bottom) showing
- [x] System Dashboard (if configured)

#### **Test 2.2: Interactive Creation with Full UI**
1. Double-click in canvas → Node created
2. Drag NUCLEUS → new node → Edge created
3. Verify panels update with new counts

#### **Test 2.3: Hot-Swap Scenarios**
1. **Action**: Close app, restart with `paint-simple.json`
2. **Expected**: UI immediately shows minimal layout
3. **Verify**: No recompilation needed, instant change

---

## 🧪 Test Scenario 3: Capability Validation

**Purpose**: Verify intelligent edge validation

### **Test Steps**:

#### **Test 3.1: Create Provider → Consumer Edge**
1. **Setup**: 
   - Create node A (imagine it's a "security-provider")
   - Create node B (imagine it's a "security-consumer")
2. **Action**: Drag A → B
3. **Expected** (terminal):
   ```
   ⚠️ Connection warning: No explicit capability match...
   ```
   (Since we're using generic nodes, validation will warn but allow)

#### **Test 3.2: Create Self-Loop**
1. **Action**: Drag node A → itself
2. **Expected** (terminal):
   ```
   ⚠️ Connection warning: Self-connection detected
   ```

#### **Test 3.3: Coordination Primal**
1. **Setup**: NUCLEUS node (has "coordination" capability)
2. **Action**: Drag NUCLEUS → any other node
3. **Expected** (terminal):
   ```
   ✅ Connection validated
   ```
   (Coordination primals can connect to anything)

---

## 🧪 Test Scenario 4: Modular UI Control

**Purpose**: Verify JSON configuration controls UI

### **Test Steps**:

#### **Test 4.1: Modify Scenario JSON**
1. **Edit**: `sandbox/scenarios/paint-simple.json`
2. **Change**:
   ```json
   "show_panels": {
     "graph_statistics": true  // Change false → true
   }
   ```
3. **Restart**: App with modified scenario
4. **Expected**: Graph Statistics panel now visible
5. **Verify**: Panel appears without code changes

#### **Test 4.2: Toggle Features**
1. **Edit**: `sandbox/scenarios/full-dashboard.json`
2. **Change**:
   ```json
   "features": {
     "audio_sonification": false  // Disable audio
   }
   ```
3. **Restart**: App
4. **Expected**: Audio processing disabled
5. **Verify**: No audio-related UI/processing

---

## ✅ Success Criteria

### **Interactive Features**:
- [x] Double-click creates nodes
- [x] Drag creates edges with blue preview line
- [x] Delete removes nodes and connected edges
- [x] Pan/zoom navigation works smoothly
- [x] No crashes or errors

### **Capability Validation**:
- [x] Terminal shows validation messages
- [x] Self-loops give warnings
- [x] Coordination primals always valid
- [x] Non-matching capabilities warn but allow

### **Modular UI**:
- [x] `paint-simple.json` shows minimal UI
- [x] `full-dashboard.json` shows all panels
- [x] Changing JSON immediately affects UI (on restart)
- [x] No recompilation needed

### **Quality**:
- [x] No console errors (check browser/terminal)
- [x] Smooth performance
- [x] Visual feedback is clear
- [x] UI is responsive

---

## 🐛 Known Issues / Expected Behavior

### **1. Canvas Background**
- **Behavior**: Canvas may show grid or solid color
- **Expected**: Configurable in future versions
- **Workaround**: None needed, cosmetic only

### **2. Node Labels**
- **Behavior**: Auto-generated IDs like "Node-4"
- **Expected**: Future: editable labels
- **Workaround**: Functional for testing

### **3. Edge Styling**
- **Behavior**: All edges look similar (gray lines)
- **Expected**: Future: capability-based colors
- **Workaround**: Edges are functional

### **4. Validation Messages**
- **Behavior**: Generic nodes will show warnings
- **Expected**: Real primals with capabilities will validate correctly
- **Workaround**: Check terminal for messages

---

## 📝 Testing Checklist

### **Basic Functionality**:
- [ ] App starts without errors
- [ ] Initial nodes render correctly
- [ ] UI layout matches scenario config
- [ ] Navigation (pan/zoom) works

### **Interactive Creation**:
- [ ] Double-click creates node
- [ ] Drag creates edge
- [ ] Blue preview line visible
- [ ] Edges snap to target nodes

### **Interactive Deletion**:
- [ ] Click selects node
- [ ] Delete key removes node
- [ ] Connected edges also removed
- [ ] No orphaned edges

### **Validation**:
- [ ] Terminal shows validation messages
- [ ] Warnings logged for non-matching capabilities
- [ ] Success logged for coordination primals
- [ ] Self-loops warn

### **Modular UI**:
- [ ] Paint mode shows minimal UI
- [ ] Full dashboard shows all panels
- [ ] Graph statistics toggleable
- [ ] Sidebars toggleable

### **Performance**:
- [ ] No lag when creating nodes
- [ ] No lag when creating edges
- [ ] Smooth pan/zoom
- [ ] No memory leaks (long session)

---

## 🎓 Advanced Testing

### **Stress Test**:
```
1. Create 20 nodes via double-click
2. Connect all nodes randomly
3. Delete half the nodes
4. Verify performance remains smooth
```

### **Edge Cases**:
```
1. Create edge from node to itself (self-loop)
2. Create duplicate edges (should prevent)
3. Delete node with many edges (should clean up)
4. Zoom way in/out (extreme zoom levels)
```

### **Scenario Switching**:
```
1. Start with paint-simple.json
2. Stop app (Ctrl+C)
3. Start with full-dashboard.json
4. Verify completely different UI
5. Repeat without recompiling
```

---

## 🚀 What to Report

### **If Something Works** ✅:
```
Feature: [Interactive Node Creation]
Status: ✅ WORKING
Details: Double-clicked canvas, node appeared instantly at cursor
Notes: Smooth, no lag, exactly as expected
```

### **If Something Doesn't Work** ❌:
```
Feature: [Interactive Edge Creation]
Status: ❌ BROKEN
Details: Dragged from node A to B, but no edge appeared
Error: [paste any terminal error]
Steps to Reproduce:
  1. Start with paint-simple.json
  2. Double-click to create node
  3. Drag from Alpha to new node
Expected: Blue line preview, then edge
Actual: Nothing happened
```

---

## 🎉 After Testing

### **If Everything Works**:
1. Celebrate! 🎉
2. Start designing real biomeOS ecosystems
3. Experiment with custom scenarios
4. Prepare for biomeOS integration

### **Next Features to Explore**:
1. Save interactive scenarios to JSON
2. Node property editor (right-click)
3. Tool palette for node types
4. Undo/redo system
5. Keyboard shortcuts

---

## 📚 Reference

### **Key Files**:
- `crates/petal-tongue-graph/src/visual_2d.rs` - Interactive canvas logic
- `crates/petal-tongue-graph/src/capability_validator.rs` - Validation logic
- `crates/petal-tongue-ui/src/scenario.rs` - Scenario loading
- `crates/petal-tongue-ui/src/app.rs` - Main app logic

### **Test Files**:
- `crates/petal-tongue-ui/tests/scenario_tests.rs` - 16 scenario tests
- `crates/petal-tongue-graph/src/capability_validator.rs` - 5 validation tests

### **Documentation**:
- `SESSION_SUMMARY_FINAL_JAN_15_2026.md` - Complete session summary
- `INTERACTIVE_PAINT_MODE_JAN_15_2026.md` - Interactive features guide
- `MODULAR_UI_COMPLETE_JAN_15_2026.md` - UI modularity details
- `DEPLOYMENT_COMPLETE_JAN_15_2026.md` - Deployment details

---

## 🌸 Happy Testing!

**Remember**: This is a TRUE PRIMAL platform now!
- No hardcoded types
- Capability-based validation
- Live evolution via JSON
- Interactive modeling

**Have fun designing ecosystems!** 🚀

---

**Version**: v2.3.0  
**Commit**: e97704b  
**Last Updated**: January 15, 2026  
**Status**: ✅ Ready for Interactive Testing

