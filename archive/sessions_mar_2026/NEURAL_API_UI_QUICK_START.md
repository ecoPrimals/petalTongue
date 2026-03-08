# 🚀 Neural API UI Quick Start

**Status**: ✅ Ready for Testing  
**Date**: January 15, 2026

---

## ⚡ **Quick Test (5 minutes)**

### **With biomeOS Running**

```bash
# Terminal 1: Start biomeOS Neural API
cd ~/Development/ecoPrimals/phase2/biomeOS
cargo run --bin nucleus -- serve --family nat0

# Terminal 2: Start petalTongue
cd ~/Development/ecoPrimals/phase2/petalTongue
cargo run --bin petal-tongue ui

# In petalTongue UI:
# Press 'P' → Opens Proprioception Panel
# Press 'M' → Opens Metrics Dashboard
# Or use: View menu → Select panels
```

### **Without biomeOS (Graceful Degradation)**

```bash
# Just run petalTongue
cargo run --bin petal-tongue ui

# Panels will show "Neural API not available"
# Core UI still works (topology, controls, etc.)
```

---

## 🎹 **Keyboard Shortcuts**

| Key | Panel |
|-----|-------|
| `P` | 🧠 Proprioception Panel |
| `M` | 📊 Metrics Dashboard |

---

## 📊 **What You'll See**

### **Proprioception Panel (P)**
- **Health Status**: Healthy/Degraded/Critical with color
- **Confidence**: 0-100% progress bar
- **SAME DAVE Breakdown**:
  - Sensory: Active sockets
  - Awareness: Known primals
  - Motor: Capabilities
  - Evaluative: System status

### **Metrics Dashboard (M)**
- **CPU Usage**: Live % + sparkline
- **Memory Usage**: Bar + sparkline
- **Uptime**: Human-readable
- **Neural API Stats**: Active primals, graphs, executions

---

## 🔍 **Verify It Works**

### **Check Logs**
```bash
# Should see:
✅ Neural API connected - using as primary provider
Fetching proprioception data from Neural API...
Proprioception data received: Health: Healthy, Confidence: 100%
```

### **Visual Checks**
- [ ] Panels open with `P` and `M` keys
- [ ] Data updates every 5 seconds
- [ ] Sparklines show history
- [ ] Health status shows color-coded emoji
- [ ] View menu has panel toggles

---

## 🐛 **Troubleshooting**

### **"Neural API not available"**
```bash
# Check if biomeOS is running:
ps aux | grep nucleus

# Check socket exists:
ls -la /tmp/biomeos-neural-api-*.sock
# or
ls -la $XDG_RUNTIME_DIR/biomeos-neural-api-*.sock
```

### **Panels Don't Update**
- Wait 5 seconds (auto-refresh throttle)
- Check biomeOS logs for errors
- Verify primals are running (beardog, songbird, toadstool)

---

## 📚 **Full Documentation**

See: [`NEURAL_API_UI_INTEGRATION_COMPLETE.md`](NEURAL_API_UI_INTEGRATION_COMPLETE.md)

---

**Happy testing!** 🌸✨

