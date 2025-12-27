# 🔍 Capability Detection - Know Thyself

**petalTongue discovers and reports its own capabilities**

---

## 🎯 What You'll Learn

In **5 minutes**:
- Self-awareness system
- Capability detection
- "Never claim false capabilities"
- Honest reporting

---

## ⏱️ Duration

**5 minutes**

---

## 📋 Prerequisites

- petalTongue built
- Completed: 00-hello through 05-dual-modality

---

## 🚀 Run the Demo

```bash
./demo.sh
```

---

## 🎓 Self-Awareness Philosophy

### **The Prime Directive**

> **"Never claim a capability that isn't real."**

petalTongue **detects** rather than **assumes**:
- ✓ Visual renderer working? → Report "visual available"
- ✓ Audio system accessible? → Report "audio available"
- ✗ System lacks audio? → Report "audio unavailable"

**Honest over optimistic!**

---

## 🔍 What Gets Detected?

### **Modalities**

| Modality | Check | If Available | If Unavailable |
|----------|-------|--------------|----------------|
| **Visual** | egui + display | Report available | Cannot run (required) |
| **Audio** | Audio device | Enable feature | Gracefully disable |
| **Haptic** | Future | Future | Currently unavailable |

### **Features**

| Feature | Check | Report |
|---------|-------|--------|
| **Animation** | GPU/CPU | "Smooth" / "Simple" / "Disabled" |
| **BingoCube** | Integration | "Available" / "Not configured" |
| **ToadStool** | Python runtime | "Available" / "Python missing" |

### **System Resources**

| Resource | Detection | Impact |
|----------|-----------|--------|
| **Display** | Resolution | Layout scaling |
| **CPU** | Core count | Animation complexity |
| **Memory** | Available RAM | Graph size limits |

---

## 👀 What You'll See

### **Capability Report (UI)**

```
┌─ System Capabilities ───────────┐
│ Visual: ✓ Available (60 FPS)   │
│ Audio: ✓ Available (44.1kHz)   │
│ Animation: ✓ Smooth             │
│                                  │
│ Integrations:                   │
│ • BingoCube: ✓ Connected        │
│ • ToadStool: ✗ Not configured   │
│                                  │
│ Performance:                     │
│ • CPU: 4 cores detected         │
│ • Memory: 8.2 GB available      │
│ • Display: 1920×1080            │
└──────────────────────────────────┘
```

### **Capability Report (API)**

```json
{
  "modalities": {
    "visual": {
      "available": true,
      "renderer": "egui",
      "fps": 60,
      "resolution": [1920, 1080]
    },
    "audio": {
      "available": true,
      "sample_rate": 44100,
      "channels": 2
    }
  },
  "features": {
    "animation": "smooth",
    "bingocube": true,
    "toadstool": false
  },
  "system": {
    "cpu_cores": 4,
    "memory_mb": 8192,
    "os": "Linux"
  }
}
```

---

## 💡 Why This Matters

### **1. Honest Communication**

Don't lie to users:
- ✗ BAD: Claim audio works, crash when used
- ✓ GOOD: Report "audio unavailable", disable gracefully

### **2. Graceful Degradation**

Adapt to available resources:
- Full GPU? → Smooth animations
- CPU only? → Simple animations
- Low memory? → Smaller graph limits

### **3. Debugging Support**

When issues occur:
- "Audio not working" → Check capability report
- Shows exactly what was detected
- Helps diagnose problems quickly

### **4. Runtime Discovery**

No hardcoded assumptions:
- Don't assume BingoCube is at port 8080
- Don't assume Python is at `/usr/bin/python3`
- Discover and adapt

---

## 📊 What This Demonstrates

1. ✅ **Self-Awareness** - Know what you can do
2. ✅ **Honest Reporting** - Never lie about capabilities
3. ✅ **Graceful Degradation** - Work with what's available
4. ✅ **Runtime Discovery** - No hardcoded assumptions
5. ✅ **User Transparency** - Show what's working

---

## 🧮 Technical Implementation

### **Capability Detector**

```rust
pub struct CapabilityDetector {
    modalities: HashMap<Modality, CapabilityStatus>,
}

impl CapabilityDetector {
    pub fn detect_all() -> Self {
        let mut detector = Self::new();
        
        // Visual (required)
        detector.detect_visual();
        
        // Audio (optional)
        detector.detect_audio();
        
        // Integrations (optional)
        detector.detect_integrations();
        
        detector
    }
    
    fn detect_audio(&mut self) {
        match AudioDevice::try_init() {
            Ok(device) => {
                self.modalities.insert(
                    Modality::Audio,
                    CapabilityStatus::Available {
                        details: device.info(),
                    }
                );
            }
            Err(e) => {
                self.modalities.insert(
                    Modality::Audio,
                    CapabilityStatus::Unavailable {
                        reason: e.to_string(),
                    }
                );
            }
        }
    }
}
```

### **Capability Status**

```rust
pub enum CapabilityStatus {
    Available { details: String },
    Unavailable { reason: String },
    Degraded { issue: String },
}
```

---

## 🎯 Real-World Scenarios

### **Scenario 1: Server Environment**

```
System: Headless server, no audio device
Detection: Audio unavailable
Behavior: Disable audio controls, work perfectly via visual
Result: ✓ Functional
```

### **Scenario 2: Low-End Hardware**

```
System: 2 core CPU, 2 GB RAM
Detection: Limited resources
Behavior: Simple animations, smaller graph limits
Result: ✓ Responsive
```

### **Scenario 3: Missing Integration**

```
System: BingoCube not installed
Detection: Integration unavailable
Behavior: Hide BingoCube panel, explain why
Result: ✓ Clear feedback
```

### **Scenario 4: Degraded Mode**

```
System: GPU driver issue
Detection: Visual degraded
Behavior: CPU-only rendering, warn user
Result: ✓ Still works
```

---

## 🐛 Troubleshooting

### **Audio shows unavailable**

Reasons:
- No audio device connected
- Audio system not initialized
- Permissions issue

**This is expected behavior!** petalTongue continues working.

### **Animations disabled**

Reasons:
- Low CPU cores detected
- High memory pressure
- User preference

Check capability report for specifics.

---

## 🎯 Success Criteria

You've mastered capability detection when you:
- ✅ Understand "never claim false capabilities"
- ✅ Can read capability reports
- ✅ Appreciate honest communication
- ✅ See how graceful degradation works
- ✅ Know why runtime discovery matters

---

## ➡️ Next Steps

```bash
cd ../07-audio-export/
cat README.md
```

**Next**: Export audio files for offline use.

---

## 📚 Technical Details

### **Detection Order**

1. **Core Modalities** (visual, audio)
2. **System Resources** (CPU, memory, display)
3. **Optional Features** (animation, integrations)
4. **External Services** (BiomeOS, BingoCube, ToadStool)

### **Update Frequency**

- **Startup**: Full detection
- **Runtime**: Resource monitoring (every 60s)
- **On-Demand**: User requests refresh

### **Performance**

Detection overhead:
- **Startup**: ~50ms (one-time)
- **Runtime**: < 1ms (periodic checks)
- **Negligible** impact

---

## 🌟 Key Takeaway

**Self-awareness is fundamental.**

petalTongue:
- ✅ Knows what it can do
- ✅ Knows what it can't do
- ✅ Reports honestly
- ✅ Adapts gracefully
- ✅ Never lies to users

**This is responsible software design!**

---

*"Know thyself, then serve well."* 🌸

