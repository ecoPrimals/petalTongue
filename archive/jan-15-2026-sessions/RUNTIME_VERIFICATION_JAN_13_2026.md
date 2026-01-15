# Runtime Verification - Substrate-Agnostic Audio

**Date**: January 13, 2026  
**Status**: ✅ Ready for Runtime Testing  
**Purpose**: Verify audio system works in production

---

## 🎯 Verification Goals

1. **Audio Discovery**: Verify AudioManager discovers available backends
2. **Backend Selection**: Verify correct backend is auto-selected
3. **Graceful Degradation**: Verify silent mode when no audio
4. **Startup Audio**: Verify signature tone plays
5. **App Integration**: Verify app initializes correctly

---

## 📋 Verification Checklist

### Build Status ✅
```bash
$ cargo build --release --package petal-tongue-ui
    Finished `release` profile [optimized] target(s) in 25.51s
✅ Release build successful
```

### Test Status ✅
```bash
$ cargo test --package petal-tongue-ui --lib audio
test result: ok. 38 passed; 0 failed; 1 ignored
✅ All audio tests passing
```

### Code Status ✅
- **9 new files**: Audio module complete
- **1,166 lines**: Substrate-agnostic implementation
- **6 files modified**: App integration complete
- **0 errors**: Clean build
- **10 documents**: Comprehensive documentation

---

## 🧪 Runtime Test Plan

### Test 1: Audio Discovery (Unit Test)
```bash
cargo test --package petal-tongue-ui --lib \
  audio::manager::tests::test_audio_manager_init \
  -- --nocapture
```

**Expected Output**:
```
🎵 Discovering audio backends (TRUE PRIMAL)...
🔌 Checking for socket-based audio servers...
✅ Socket audio server: Socket Audio (PipeWire)
🎨 Checking for direct audio devices...
✅ Direct audio device: Direct Audio Device (Pcm)
🎼 Pure Rust software synthesis available
🔇 Silent mode available (graceful degradation)
🎵 Found 4 audio backend(s)
```

### Test 2: Backend Selection
```bash
cargo test --package petal-tongue-ui --lib \
  audio::compat::tests::test_audio_system_v2_creation \
  -- --nocapture
```

**Expected Output**:
```
Available backends:
  - Socket Audio (PipeWire) (Socket)
  - Direct Audio Device (Pcm) (Direct)
  - Pure Rust Software Synthesis (Software)
  - Silent Mode (Silent)
```

### Test 3: Audio Playback
```bash
cargo test --package petal-tongue-ui --lib \
  audio::compat::tests::test_audio_system_v2_play_tone \
  -- --nocapture
```

**Expected**: No panic, graceful handling

### Test 4: Full App Startup (Manual)
```bash
cd /path/to/petalTongue
RUST_LOG=info cargo run --release --package petal-tongue-ui
```

**Expected Log Output**:
```
🎵 Initializing AudioSystemV2 (substrate-agnostic)...
🎵 Discovering audio backends (TRUE PRIMAL)...
🔌 Checking for socket-based audio servers...
✅ Socket audio server: Socket Audio (PipeWire)
...
🎵 Active audio backend: Socket Audio (PipeWire)
🎵 Available audio backends: 4 total
  - Socket Audio (PipeWire) (Socket)
  - Direct Audio Device (Pcm) (Direct)
  - Pure Rust Software Synthesis (Software)
  - Silent Mode (Silent)
🎵 Initializing startup audio...
🎵 Starting petalTongue startup audio...
```

---

## 🌐 Platform-Specific Expectations

### On Linux with PipeWire
```
Expected Selection: SocketBackend (PipeWire)
Priority: Tier 2 (highest available)
Behavior: Audio plays through PipeWire
```

### On Linux with PulseAudio
```
Expected Selection: SocketBackend (PulseAudio)
Priority: Tier 2 (highest available)  
Behavior: Audio plays through PulseAudio
```

### On Linux with ALSA Only
```
Expected Selection: DirectBackend (/dev/snd)
Priority: Tier 2
Behavior: Direct hardware access
```

### On Headless Server
```
Expected Selection: SilentBackend
Priority: Tier 5 (graceful degradation)
Behavior: No audio output, no errors
```

### On macOS
```
Expected Selection: SocketBackend or DirectBackend (/dev/audio)
Priority: Tier 2-3
Behavior: Platform-appropriate audio
```

### On Windows
```
Expected Selection: SoftwareBackend or SilentBackend
Priority: Tier 4-5
Behavior: Graceful degradation
```

---

## ✅ Success Criteria

### Functional Requirements
- [ ] AudioManager initializes without panic
- [ ] Backend discovery completes
- [ ] Active backend is selected
- [ ] All backends are enumerated
- [ ] Startup audio plays (or degrades gracefully)
- [ ] App starts successfully
- [ ] No errors in logs

### Performance Requirements
- [ ] Initialization < 100ms
- [ ] No blocking on main thread
- [ ] Audio playback non-blocking

### Reliability Requirements
- [ ] Works on systems with audio
- [ ] Works on systems without audio
- [ ] Graceful degradation everywhere
- [ ] No panics or crashes

---

## 🔍 Debugging Commands

### Check System Audio
```bash
# Check for PipeWire
ls -la /run/user/$(id -u)/pipewire-0

# Check for PulseAudio
ls -la /run/user/$(id -u)/pulse/native

# Check for ALSA devices
ls -la /dev/snd/

# Check running audio services
ps aux | grep -E "(pipewire|pulseaudio)"
```

### Run with Debug Logging
```bash
RUST_LOG=debug cargo run --package petal-tongue-ui 2>&1 | grep -E "(🎵|audio|backend)"
```

### Test Individual Backends
```bash
# Test socket discovery
cargo test --package petal-tongue-ui --lib \
  audio::backends::socket::tests::test_socket_discovery \
  -- --nocapture

# Test direct device discovery  
cargo test --package petal-tongue-ui --lib \
  audio::backends::direct::tests::test_direct_discovery \
  -- --nocapture
```

---

## 📊 Expected Runtime Behavior

### Initialization Sequence

1. **AudioSystemV2::new()** called
   - Gets or creates tokio runtime
   - Calls AudioManager::init() async
   
2. **AudioManager::init()** discovers backends
   - Socket discovery (PipeWire/PulseAudio)
   - Direct device discovery (/dev/snd, etc.)
   - Software synthesis (always available)
   - Silent mode (always available)
   
3. **Backend selection** on first use
   - Sorts by priority
   - Tries each backend
   - Selects first available

4. **Audio playback** through selected backend
   - Converts samples to backend format
   - Non-blocking write
   - Error handling with fallback

---

## 🎯 Key Verification Points

### 1. No Hardcoding ✅
```bash
# Should find NO hardcoded OS APIs
grep -r "CoreAudio\|WASAPI\|alsa-sys" \
  crates/petal-tongue-ui/src/audio/
# Expected: No matches
```

### 2. Runtime Discovery ✅
```bash
# Should show dynamic discovery
cargo test --lib audio::manager::tests::test_audio_manager_init \
  -- --nocapture 2>&1 | grep "Discovered\|available"
# Expected: Backend discovery logs
```

### 3. Graceful Degradation ✅
```bash
# Should work even in minimal environment
env -i cargo test --lib audio::backends::silent::tests
# Expected: All tests pass
```

---

## 🌟 Next Steps

### Immediate (Today)
1. Run unit tests with --nocapture to see discovery logs
2. Verify backend selection on this system
3. Test audio playback (if available)

### Short-term (This Week)
1. Test on different Linux distros
2. Test with/without audio hardware
3. Test biomeOS installer (headless mode)

### Medium-term (Next Week)
1. Test on macOS
2. Test on Windows
3. Test on embedded devices

---

## 📝 Testing Log

### Test Session: January 13, 2026

**System**: Linux (your current system)

**Audio Available**:
- [ ] PipeWire: ___________
- [ ] PulseAudio: ___________
- [ ] Direct ALSA: ___________
- [ ] None (headless): ___________

**Tests Run**:
- [ ] Unit tests passing
- [ ] Backend discovery working
- [ ] Audio playback tested
- [ ] App startup successful

**Results**:
- Selected Backend: ___________
- Audio Working: ___________
- Startup Sound: ___________
- Notes: ___________

---

## ✅ Completion Checklist

- [x] Code complete (9 files, 1,166 lines)
- [x] Tests passing (38 tests)
- [x] Build clean (release mode)
- [x] Documentation complete (10 docs)
- [x] Integration complete (app updated)
- [ ] Runtime tested (manual verification)
- [ ] Audio confirmed working
- [ ] Platform testing complete

---

🌸 **Ready for runtime verification - the moment of truth!** 🌸

**Status**: ✅ All pre-runtime checks complete  
**Next**: Manual testing and verification

---

*Let's hear that substrate-agnostic audio!* 🎵

