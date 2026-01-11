# 🏗️ Build Requirements

**petalTongue Build Dependencies**

---

## 🎯 Overview

petalTongue is **100% Pure Rust** and self-contained. However, some build-time dependencies are needed for compilation (not runtime).

**Runtime**: petalTongue binary has ZERO external dependencies after compilation!

---

## 📦 Linux (Ubuntu/Debian)

### **Audio Support** (Required for rodio/cpal)

```bash
sudo apt-get update
sudo apt-get install -y libasound2-dev pkg-config
```

**Why?** `cpal` (cross-platform audio I/O) needs ALSA headers at compile time. The compiled binary will auto-select the best audio backend at runtime (PulseAudio, JACK, ALSA, etc.).

### **Full Build Environment**

```bash
# Rust toolchain (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build dependencies
sudo apt-get install -y \
  build-essential \
  pkg-config \
  libasound2-dev

# Optional: For development
sudo apt-get install -y git
```

---

## 📦 Linux (Fedora/RHEL)

```bash
# Audio support
sudo dnf install -y alsa-lib-devel pkg-config

# Full build environment
sudo dnf install -y \
  gcc \
  alsa-lib-devel \
  pkg-config
```

---

## 📦 Linux (Arch)

```bash
# Audio support
sudo pacman -S alsa-lib pkg-config

# Full build environment
sudo pacman -S base-devel alsa-lib pkg-config
```

---

## 📦 macOS

```bash
# Install Homebrew (if not installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# No additional dependencies needed!
# macOS CoreAudio is used automatically
```

---

## 📦 Windows

```bash
# Install Visual Studio C++ Build Tools
# Download from: https://visualstudio.microsoft.com/downloads/

# Or install via winget
winget install Microsoft.VisualStudio.2022.BuildTools

# No additional audio dependencies needed!
# Windows WASAPI is used automatically
```

---

## ✅ Verification

### **Check Rust Installation**

```bash
rustc --version
cargo --version
```

### **Check Build Dependencies** (Linux)

```bash
# ALSA headers
pkg-config --exists alsa && echo "✅ ALSA OK" || echo "❌ ALSA missing"

# pkg-config
which pkg-config && echo "✅ pkg-config OK" || echo "❌ pkg-config missing"
```

### **Test Build**

```bash
cd /path/to/petalTongue
cargo build --release
```

**Success**: Binary at `target/release/petal-tongue`

---

## 🎯 Build vs Runtime

| Component | Build Time | Runtime |
|-----------|------------|---------|
| **ALSA headers** | Required (Linux) | Not required |
| **pkg-config** | Required (Linux) | Not required |
| **Visual Studio C++** | Required (Windows) | Not required |
| **Audio players** | Not required | Not required ✅ |
| **X11/Wayland** | Not required | Auto-detected ✅ |
| **Display tools** | Not required | Not required ✅ |

**After compilation**: petalTongue binary is **completely self-contained**!

---

## 🐛 Troubleshooting

### **Error**: `alsa-sys` build failed

```bash
# Install ALSA development headers
sudo apt-get install libasound2-dev pkg-config  # Ubuntu/Debian
sudo dnf install alsa-lib-devel pkg-config       # Fedora/RHEL
sudo pacman -S alsa-lib pkg-config               # Arch
```

### **Error**: `pkg-config` not found

```bash
# Install pkg-config
sudo apt-get install pkg-config  # Ubuntu/Debian
sudo dnf install pkg-config       # Fedora/RHEL
sudo pacman -S pkg-config         # Arch
```

### **Error**: Linker errors on Windows

```bash
# Install Visual Studio C++ Build Tools
# https://visualstudio.microsoft.com/downloads/
```

---

## 📚 Platform-Specific Notes

### **Linux**

- **ALSA**: Compile-time only, runtime uses best available (PulseAudio preferred)
- **X11/Wayland**: Auto-detected at runtime via `egui`/`winit`
- **Display**: Works headless, X11, Wayland, framebuffer

### **macOS**

- **CoreAudio**: Built-in, no dependencies
- **Display**: Works natively with Cocoa

### **Windows**

- **WASAPI**: Built-in, no dependencies
- **Display**: Works natively with Win32

---

## ✨ TRUE PRIMAL Sovereignty

**Build Requirements** ≠ **Runtime Dependencies**

- **Build**: Need platform development headers (standard practice)
- **Runtime**: petalTongue is **completely self-contained** ✅

**After compilation**, you can deploy the `petal-tongue` binary to any system without any dependencies!

---

## 🚀 Quick Start

### **Ubuntu/Debian**

```bash
# One-time setup
sudo apt-get install -y libasound2-dev pkg-config

# Build
cargo build --release

# Run
./target/release/petal-tongue
```

### **macOS**

```bash
# No setup needed!
cargo build --release
./target/release/petal-tongue
```

### **Windows**

```powershell
# Install Visual Studio C++ Build Tools (one-time)
# Then:
cargo build --release
.\target\release\petal-tongue.exe
```

---

**Date**: January 11, 2026  
**Status**: Active  
**Purpose**: Document build requirements for TRUE PRIMAL sovereignty

