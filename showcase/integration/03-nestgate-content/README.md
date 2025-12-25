# NestGate Content Fingerprints with BingoCube

**Primal**: NestGate (Storage & Content Addressing)  
**Use Case**: Visual git commits and content recognition  
**Duration**: 10 minutes

---

## Scenario

You're working with content stored in NestGate. Instead of memorizing hex hashes, you **recognize content by its BingoCube pattern**—like recognizing a friend's face rather than their ID number.

### Use Cases

1. **Visual Git Commits**: Recognize commits by pattern
2. **Content Verification**: Quickly verify file integrity
3. **Redundancy Status**: See storage redundancy level
4. **Provenance Tracking**: Visual audit trail

---

## Demo 1: Visual Git Commits

Traditional git:
```
commit 7f3a2b1c... "Add BingoCube support"
commit 9e8c4d2f... "Fix layout bug"
commit a1b4c8e3... "Update docs"
```

**Problem**: Hex hashes are unmemorable.

With BingoCube:
```
Commit "Add BingoCube":     Commit "Fix layout":       Commit "Update docs":
┌───────────┐              ┌───────────┐              ┌───────────┐
│ 🟦 🟥 🟨 🟩 🟦│              │ 🟥 🟨 🟦 🟩 🟥│              │ 🟩 🟦 🟥 🟨 🟩│
│ 🟩 🟦 🟥 🟨 🟩│              │ 🟦 🟩 🟨 🟦 🟨│              │ 🟨 🟥 🟩 🟦 🟨│
│ 🟨 🟥 ✱ 🟦 🟨│              │ 🟩 🟦 ✱ 🟥 🟩│              │ 🟦 🟩 ✱ 🟨 🟦│
│ 🟦 🟩 🟨 🟥 🟦│              │ 🟨 🟥 🟩 🟨 🟦│              │ 🟥 🟨 🟦 🟩 🟥│
│ 🟥 🟨 🟦 🟩 🟥│              │ 🟦 🟨 🟥 🟩 🟥│              │ 🟨 🟦 🟩 🟥 🟨│
└───────────┘              └───────────┘              └───────────┘
```

**Benefit**: "Oh, that's the BingoCube commit!" (instant recognition)

---

## Demo 2: Content Fingerprints

### Boards
```
Board A (Content Hash):
  Seed: BLAKE3(file_contents)
  Fixed: Permanent for this content version
  
Board B (Metadata Hash):
  Seed: BLAKE3(size || mime_type || timestamp || permissions)
  Variable: Changes if metadata changes
```

### Same Content, Different Metadata

```
original.pdf                 same_content_renamed.pdf
(uploaded today)             (uploaded tomorrow)

Board A: Same                Board A: Same (identical)
Board B: Different           Board B: Different (timestamp changed)

Result: Similar patterns, but not identical
→ Visual indication of "same content, different context"
```

---

## Demo 3: Redundancy Visualization

Use **x parameter to show redundancy level**:

**Low Redundancy** (x=0.3):
```
┌───────────┐
│ · · 🟥 · ·│
│ · 🟦 · 🟨 ·│
│ 🟩 · ✱ · ·│
│ · · · 🟦 ·│
│ · 🟨 🟩 · ·│
└───────────┘
⚠️ Warning: Only 30% redundancy
Risk of data loss!
```

**Good Redundancy** (x=0.7):
```
┌───────────┐
│ 🟦 🟩 🟥 🟨 ·│
│ 🟨 🟦 🟩 🟨 🟥│
│ 🟩 🟥 ✱ 🟦 🟨│
│ 🟦 🟩 🟨 🟦 🟩│
│ 🟥 🟨 🟩 · 🟦│
└───────────┘
✅ Safe: 70% redundancy
Multiple copies available
```

**Maximum Redundancy** (x=1.0):
```
┌───────────┐
│ 🟦 🟩 🟥 🟨 🟦│
│ 🟨 🟦 🟩 🟨 🟥│
│ 🟩 🟥 ✱ 🟦 🟨│
│ 🟦 🟩 🟨 🟦 🟩│
│ 🟥 🟨 🟩 🟥 🟦│
└───────────┘
💎 Perfect: Full redundancy
Maximum data safety
```

---

## Demo 4: Provenance Chain

Track content through its lifecycle:

```
1. Created              2. Stored              3. Modified            4. Archived
┌───────────┐          ┌───────────┐          ┌───────────┐          ┌───────────┐
│ 🟦 🟩 🟥 🟨 🟦│          │ 🟦 🟩 🟥 🟨 🟦│          │ 🟥 🟨 🟦 🟩 🟥│          │ 🟩 🟦 🟥 🟨 🟩│
│ 🟨 🟦 🟩 🟨 🟥│          │ 🟨 🟦 🟩 🟨 🟥│          │ 🟦 🟩 🟨 🟦 🟨│          │ 🟨 🟥 🟩 🟦 🟨│
│ 🟩 🟥 ✱ 🟦 🟨│          │ 🟩 🟥 ✱ 🟦 🟨│          │ 🟩 🟦 ✱ 🟥 🟩│          │ 🟦 🟩 ✱ 🟨 🟦│
│ 🟦 🟩 🟨 🟦 🟩│          │ 🟦 🟩 🟨 🟦 🟩│          │ 🟨 🟥 🟩 🟨 🟦│          │ 🟥 🟨 🟦 🟩 🟥│
│ 🟥 🟨 🟩 🟥 🟦│          │ 🟥 🟨 🟩 🟥 🟦│          │ 🟦 🟨 🟥 🟩 🟥│          │ 🟨 🟦 🟩 🟥 🟨│
└───────────┘          └───────────┘          └───────────┘          └───────────┘
v1.0 (original)        v1.0 (same)            v2.0 (edited)          v2.0 (archived)

Board A: Same          Board A: Same          Board A: Different     Board A: Different
Board B: timestamp1    Board B: timestamp1    Board B: timestamp2    Board B: archive_meta
```

**Visual History**: See content evolution at a glance.

---

## Code Example

```rust
use bingocube_core::{BingoCube, Config};

struct ContentFingerprint {
    content_hash: [u8; 32],
    metadata: ContentMetadata,
    redundancy_level: f64,
}

struct ContentMetadata {
    size: u64,
    mime_type: String,
    timestamp: SystemTime,
    permissions: Permissions,
}

impl ContentFingerprint {
    fn generate_bingocube(&self) -> Result<BingoCube> {
        // Board A: Content hash (permanent)
        let board_a_seed = self.content_hash;
        
        // Board B: Metadata hash (variable)
        let mut meta_bytes = Vec::new();
        meta_bytes.extend_from_slice(&self.metadata.size.to_le_bytes());
        meta_bytes.extend_from_slice(self.metadata.mime_type.as_bytes());
        // ... add timestamp, permissions, etc.
        let board_b_seed = blake3::hash(&meta_bytes);
        
        // Composite seed
        let mut seed = Vec::new();
        seed.extend_from_slice(&board_a_seed);
        seed.extend_from_slice(board_b_seed.as_bytes());
        
        BingoCube::from_seed(&seed, Config::default())
    }
    
    fn visualize_with_redundancy(&self) -> Result<()> {
        let cube = self.generate_bingocube()?;
        let subcube = cube.subcube(self.redundancy_level)?;
        
        println!("Content: {}", hex::encode(self.content_hash));
        println!("Redundancy: {:.0}% ({}/25 cells)", 
                 self.redundancy_level * 100.0,
                 subcube.revealed_count());
        
        if self.redundancy_level < 0.5 {
            println!("⚠️  WARNING: Low redundancy!");
        }
        
        // Render visual...
        Ok(())
    }
}
```

---

## Practical Applications

### 1. Visual File Browser
```
Instead of:                 Show:
document.pdf (7f3a2b1...)   document.pdf [🟦🟥🟨 pattern]
report.docx (9e8c4d2...)    report.docx  [🟥🟨🟦 pattern]
```

### 2. Quick Verification
```
Expected pattern:           Actual pattern:
┌───────────┐              ┌───────────┐
│ 🟦 🟩 🟥 🟨 🟦│              │ 🟦 🟩 🟥 🟨 🟦│
│ 🟨 🟦 🟩 🟨 🟥│              │ 🟨 🟦 🟩 🟨 🟥│
│ 🟩 🟥 ✱ 🟦 🟨│  ✅ Match    │ 🟩 🟥 ✱ 🟦 🟨│
│ 🟦 🟩 🟨 🟦 🟩│              │ 🟦 🟩 🟨 🟦 🟩│
│ 🟥 🟨 🟩 🟥 🟦│              │ 🟥 🟨 🟩 🟥 🟦│
└───────────┘              └───────────┘
```

### 3. Deduplication Detection
```
Two files with different names but same pattern?
→ Likely duplicates
→ Can merge storage
```

---

## Running the Demo

```bash
cd showcase/integration/03-nestgate-content
./demo.sh
```

The demo will:
1. Generate fingerprints for different files
2. Show visual git commit recognition
3. Demonstrate redundancy visualization
4. Display content provenance chain

---

## Success Criteria

You should understand:
- ✅ How content gets memorable visual identity
- ✅ Why visual commits are more usable than hex
- ✅ How redundancy becomes visible
- ✅ Why provenance tracking becomes intuitive

---

**Next**: [04 - ToadStool Computation Proofs](../04-toadstool-compute/)

