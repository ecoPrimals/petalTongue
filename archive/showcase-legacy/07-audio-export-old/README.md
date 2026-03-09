# 🎵📁 Audio Export - Sonification to File

**Export graph sonifications as audio files**

---

## 🎯 What You'll Learn

In **5 minutes**:
- Audio file generation (pure Rust WAV)
- Sonification export workflow
- Offline playback
- Cross-platform compatibility

---

## ⏱️ Duration

**5 minutes**

---

## 📋 Prerequisites

- petalTongue built
- Completed: 00-hello through 06-capability-detection

---

## 🚀 Run the Demo

```bash
./demo.sh
```

---

## 🎓 Audio Export System

### **What It Does**

Converts **graph state → audio file**:

```
Graph State
    ↓
Sonification Engine
    ↓
Pure Rust WAV Generator
    ↓
Audio File (.wav)
```

**No external dependencies!**

### **File Format**

```
Format: WAV (uncompressed)
Sample Rate: 44,100 Hz (CD quality)
Bit Depth: 16-bit
Channels: 2 (stereo)
Generator: Pure Rust (no system libs)
```

---

## 💡 Why Export Audio?

### **1. Offline Playback**

Listen without running petalTongue:
- Media players
- Mobile devices
- Sharing with others
- Archival purposes

### **2. Analysis**

Audio analysis tools:
- Spectrograms
- Frequency analysis
- Pattern recognition
- Machine learning

### **3. Accessibility**

Alternative access:
- Screen readers can announce file creation
- Standard audio players (universally accessible)
- No specialized software needed

### **4. Documentation**

System state records:
- "How did the system sound on Dec 27?"
- Audio logs for incidents
- Before/after comparisons
- Training materials

---

## 👂 What You'll Hear

### **Exported File Contains**

```
Time 0-5s: Full graph sonification
  🎐 Chimes (Songbird) - Left
  🎸 Bass (BearDog) - Center
  🎻 Strings (NestGate) - Right
  🥁 Drums (ToadStool) - Left-Center
  🎹 Synth (Squirrel) - Right-Center

Encoding:
  • Primal types → Instruments
  • Health states → Pitch quality
  • Positions → Stereo panning
  • Activity → Rhythm/intensity
```

**Snapshot of system state in audio!**

---

## 📊 What This Demonstrates

1. ✅ **Pure Rust Audio** - No system dependencies
2. ✅ **Cross-Platform** - Works everywhere
3. ✅ **Offline Capability** - Export for later
4. ✅ **Accessibility** - Standard format
5. ✅ **Documentation** - Audio records

---

## 🧮 Technical Implementation

### **WAV File Structure**

```
RIFF Header (12 bytes)
├── "RIFF" magic
├── File size
└── "WAVE" format

Format Chunk (24 bytes)
├── "fmt " chunk ID
├── Audio format (PCM = 1)
├── Channels (2 = stereo)
├── Sample rate (44100 Hz)
└── Bit depth (16-bit)

Data Chunk (variable)
├── "data" chunk ID
├── Data size
└── Audio samples (16-bit signed integers)
```

### **Pure Rust Generator**

```rust
pub struct AudioFileGenerator;

impl AudioFileGenerator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn generate_wav(
        &self,
        graph: &PrimalGraph,
        duration_secs: f32,
    ) -> Result<Vec<u8>> {
        let sample_rate = 44100;
        let samples = (sample_rate as f32 * duration_secs) as usize;
        
        let mut wav_data = Vec::new();
        
        // Write WAV header
        self.write_wav_header(&mut wav_data, samples)?;
        
        // Generate samples
        for i in 0..samples {
            let time = i as f32 / sample_rate as f32;
            let (left, right) = self.generate_sample(graph, time);
            
            // Write stereo sample (16-bit signed)
            wav_data.extend_from_slice(&left.to_le_bytes());
            wav_data.extend_from_slice(&right.to_le_bytes());
        }
        
        Ok(wav_data)
    }
}
```

**No dependencies = No breakage!**

---

## 💡 Try This

### **1. Export Audio File**

In the UI:
1. Open "Audio Export" panel
2. Set duration (default: 5 seconds)
3. Click "Export WAV"
4. Choose save location

### **2. Play in Media Player**

```bash
# Linux
vlc graph_sonification.wav

# Or any media player
```

### **3. Analyze Waveform**

```bash
# View in Audacity (if installed)
audacity graph_sonification.wav

# Or use other audio tools
```

### **4. Share**

Email, upload, or share the file.
**Anyone can listen with standard players!**

---

## 🎯 Use Cases

### **Monitoring**

```bash
# Export system state every hour
while true; do
  timestamp=$(date +%Y%m%d_%H%M%S)
  petaltongue --export-audio "system_${timestamp}.wav"
  sleep 3600
done
```

**Audio logs for system health!**

### **Comparison**

```bash
# Before deployment
petaltongue --export-audio before.wav

# After deployment  
petaltongue --export-audio after.wav

# Listen to both - hear the difference?
```

### **Accessibility**

```bash
# Generate daily report
petaltongue --export-audio daily_report.wav

# Accessible via any audio player
# Screen reader can announce: "Daily report generated"
```

---

## 🐛 Troubleshooting

### **File size large**

WAV is uncompressed:
- 5 seconds ≈ 850 KB
- 60 seconds ≈ 10 MB

**This is expected!** Can convert to MP3/OGG later if needed.

### **Playback issues**

Ensure:
- File is complete (check size)
- Player supports WAV
- Sample rate: 44.1 kHz
- Bit depth: 16-bit

### **Export fails**

Check:
- Write permissions
- Disk space
- File path valid

---

## 🎯 Success Criteria

You've mastered audio export when you:
- ✅ Can export graph as audio file
- ✅ Understand WAV format basics
- ✅ Can play exported files
- ✅ Appreciate pure Rust implementation
- ✅ See use cases for audio records

---

## ➡️ Next Steps

```bash
cd ../08-tool-integration/
cat README.md
```

**Next**: Integrate external tools dynamically.

---

## 📚 Technical Details

### **Sample Rate Choices**

| Rate | Quality | Use Case |
|------|---------|----------|
| 22.05 kHz | Voice | Podcasts |
| **44.1 kHz** | CD | **Music (our choice)** |
| 48 kHz | Professional | Video production |
| 96 kHz | High-res | Audiophiles |

We chose **44.1 kHz** for broad compatibility.

### **Bit Depth**

| Depth | Dynamic Range | Use Case |
|-------|---------------|----------|
| 8-bit | 48 dB | Retro |
| **16-bit** | 96 dB | **CD (our choice)** |
| 24-bit | 144 dB | Professional |
| 32-bit | Floating | Processing |

We chose **16-bit** for standard compatibility.

### **File Size Calculation**

```
Size = Sample_Rate × Channels × Bit_Depth × Duration
     = 44,100 Hz × 2 × 2 bytes × Duration_secs
     = 176,400 bytes/sec × Duration_secs
     
5 seconds = 882,000 bytes ≈ 860 KB
60 seconds = 10,584,000 bytes ≈ 10.1 MB
```

### **Cross-Platform**

Pure Rust WAV generation works on:
- ✅ Linux (all distros)
- ✅ macOS
- ✅ Windows
- ✅ BSD
- ✅ Any Rust target

**No system audio dependencies!**

---

## 🌟 Key Takeaway

**Audio export enables offline, accessible, shareable sonifications.**

Benefits:
- ✅ Works anywhere (standard format)
- ✅ No dependencies (pure Rust)
- ✅ Accessible (universal players)
- ✅ Archival (state records)
- ✅ Shareable (send to anyone)

**Multi-modal design extends beyond the app!**

---

*"Preserve information in every form."* 🌸

