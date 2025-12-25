# ToadStool Computation Proofs with BingoCube

**Primal**: ToadStool (Distributed Computing)  
**Use Case**: Real-time computation progress visualization  
**Duration**: 10 minutes

---

## Scenario

You submit a long-running computation to ToadStool (e.g., video encoding, ML training, data processing). Instead of waiting blindly, you **watch the computation progress** through BingoCube's progressive reveal.

### The Experience

**Before**: "Is it done yet?" (checking every 5 minutes)  
**With BingoCube**: Watch the pattern grow in real-time

---

## How It Works

### Boards
```
Board A (Input Hash):
  Seed: BLAKE3(input_data)
  Fixed: Never changes during computation
  
Board B (Output Hash):
  Seed: BLAKE3(partial_output || progress)
  Dynamic: Updates as computation proceeds
  
x Parameter = Progress (0.0 → 1.0)
```

### Visual Progress

**t=0% (Starting)**:
```
┌───────────┐
│ · · · · ·│
│ · · · · ·│
│ · · ✱ · ·│
│ · · · · ·│
│ · · · · ·│
└───────────┘
Status: Initializing...
Progress: 0/25 cells
```

**t=25% (Early Progress)**:
```
┌───────────┐
│ · · 🟥 · ·│
│ · 🟦 · · ·│
│ 🟨 · ✱ 🟩 ·│
│ · · · 🟦 ·│
│ · · · · ·│
└───────────┘
Status: Processing... (6/25)
ETA: 15 minutes
```

**t=50% (Halfway)**:
```
┌───────────┐
│ 🟦 · 🟥 🟨 ·│
│ · 🟦 🟩 · 🟥│
│ 🟨 · ✱ 🟩 🟦│
│ 🟦 🟩 · 🟦 ·│
│ · 🟨 🟩 · ·│
└───────────┘
Status: Processing... (13/25)
ETA: 10 minutes
```

**t=100% (Complete)**:
```
┌───────────┐
│ 🟦 🟩 🟥 🟨 🟦│
│ 🟨 🟦 🟩 🟨 🟥│
│ 🟩 🟥 ✱ 🟦 🟨│
│ 🟦 🟩 🟨 🟦 🟩│
│ 🟥 🟨 🟩 🟥 🟦│
└───────────┘
Status: ✅ Complete (25/25)
Result ready!
```

---

## Demo 1: Video Encoding

**Task**: Encode 4K video (2 hours → 1 hour at 60fps)

```
Input: raw_video.mp4 (50GB)
Output: encoded_video.mp4 (5GB)

Progress Timeline:
00:00 → x=0.00 (empty grid)
05:00 → x=0.08 (2 cells)
15:00 → x=0.25 (6 cells)
30:00 → x=0.50 (13 cells)
45:00 → x=0.75 (19 cells)
60:00 → x=1.00 (25 cells) ✅ Done!
```

**User Experience**: "I can see it's making progress!" (not staring at spinner)

---

## Demo 2: ML Training

**Task**: Train neural network (1000 epochs)

```
Board A: Training data hash
Board B: Model weights hash (changes per epoch)
x: epochs_completed / total_epochs

Epoch 100  → x=0.10 → ┌─────┐ 3 cells
                       │·· ·· │
                       │·🟦·· │
                       │··✱·· │
                       │····· │
                       │····· │
                       └─────┘

Epoch 500  → x=0.50 → ┌─────┐ 13 cells
                       │🟦·🟥🟨·│
                       │·🟦🟩·🟥│
                       │🟨·✱🟩🟦│
                       │🟦🟩·🟦·│
                       │·🟨🟩··│
                       └─────┘

Epoch 1000 → x=1.00 → ┌─────┐ 25 cells (complete)
                       │🟦🟩🟥🟨🟦│
                       │🟨🟦🟩🟨🟥│
                       │🟩🟥✱🟦🟨│
                       │🟦🟩🟨🟦🟩│
                       │🟥🟨🟩🟥🟦│
                       └─────┘
```

---

## Demo 3: Distributed Computation

**Task**: Process large dataset across 10 workers

```
Worker 1: Chunk 0-10%   → x=0.10 → BingoCube shows partial
Worker 2: Chunk 10-20%  → x=0.20 → More cells appear
...
Worker 10: Chunk 90-100% → x=1.00 → Complete pattern

Combined Proof:
┌───────────┐
│ 🟦 🟩 🟥 🟨 🟦│  ← Generated from HASH(all worker outputs)
│ 🟨 🟦 🟩 🟨 🟥│
│ 🟩 🟥 ✱ 🟦 🟨│
│ 🟦 🟩 🟨 🟦 🟩│
│ 🟥 🟨 🟩 🟥 🟦│
└───────────┘

Verification: Does this match expected output hash?
✅ Yes → Computation valid
❌ No  → Worker error detected
```

---

## Verification Without Re-Computation

**Key Benefit**: Verify results without redoing the work

```rust
// Requester submits job
let input_hash = blake3::hash(input_data);
let expected_cube = BingoCube::from_seed(&input_hash, Config::default())?;

// ToadStool computes
let output = heavy_computation(input_data);
let output_hash = blake3::hash(&output);

// Generate proof
let mut seed = Vec::new();
seed.extend_from_slice(input_hash.as_bytes());
seed.extend_from_slice(output_hash.as_bytes());
let proof_cube = BingoCube::from_seed(&seed, Config::default())?;

// Requester verifies
let challenge_x = random_f64(); // Random x value
let expected_subcube = expected_cube.subcube(challenge_x)?;
let proof_subcube = proof_cube.subcube(challenge_x)?;

if expected_subcube == proof_subcube {
    println!("✅ Computation verified!");
} else {
    println!("❌ Computation invalid!");
}
```

**Security**: ~2^(-50) forgery probability at x=0.5  
**Cost**: No re-computation needed!

---

## Code Example

```rust
use bingocube_core::{BingoCube, Config};
use std::time::Instant;

struct ComputationProof {
    input_hash: [u8; 32],
    output_hash: Option<[u8; 32]>,
    progress: f64,
    started_at: Instant,
}

impl ComputationProof {
    fn new(input_data: &[u8]) -> Self {
        Self {
            input_hash: *blake3::hash(input_data).as_bytes(),
            output_hash: None,
            progress: 0.0,
            started_at: Instant::now(),
        }
    }
    
    fn update_progress(&mut self, partial_output: &[u8], progress: f64) {
        self.output_hash = Some(*blake3::hash(partial_output).as_bytes());
        self.progress = progress;
    }
    
    fn generate_bingocube(&self) -> Result<BingoCube> {
        // Board A: Input (fixed)
        let board_a_seed = self.input_hash;
        
        // Board B: Output (growing)
        let board_b_seed = self.output_hash.unwrap_or([0u8; 32]);
        
        // Composite seed
        let mut seed = Vec::new();
        seed.extend_from_slice(&board_a_seed);
        seed.extend_from_slice(&board_b_seed);
        
        BingoCube::from_seed(&seed, Config::default())
    }
    
    fn visualize(&self) -> Result<()> {
        let cube = self.generate_bingocube()?;
        let subcube = cube.subcube(self.progress)?;
        
        let elapsed = self.started_at.elapsed();
        let eta = if self.progress > 0.0 {
            elapsed.mul_f64(1.0 / self.progress - 1.0)
        } else {
            Duration::from_secs(0)
        };
        
        println!("Progress: {:.0}% ({}/25 cells)", 
                 self.progress * 100.0,
                 subcube.revealed_count());
        println!("Elapsed: {:?}", elapsed);
        println!("ETA: {:?}", eta);
        
        // Render visual...
        Ok(())
    }
}
```

---

## Running the Demo

```bash
cd showcase/integration/04-toadstool-compute
./demo.sh
```

The demo will:
1. Simulate long-running computation
2. Show real-time progress visualization
3. Demonstrate verification without re-computation
4. Display distributed computation proof

---

## Multi-Modal Enhancement (Future)

### Audio Progress
- **Silence**: Not started (x=0.0)
- **Soft tones**: Early progress (x=0.3)
- **Rich melody**: Halfway (x=0.5)
- **Full chord**: Complete (x=1.0)

**Benefit**: Background monitoring while working

### Animation
- **Pulse rate**: Faster pulse = more progress
- **Flow direction**: Inward (consuming input) → outward (producing output)
- **Color intensity**: Brighter as computation proceeds

---

## Success Criteria

You should understand:
- ✅ How computation becomes visually trackable
- ✅ Why progressive reveal shows real-time progress
- ✅ How verification works without re-computation
- ✅ Why distributed computation proofs are practical

---

**Next**: [05 - Cross-Primal Workflow](../05-cross-primal/)

