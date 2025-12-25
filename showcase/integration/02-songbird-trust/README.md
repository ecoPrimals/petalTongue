# Songbird P2P Trust Stamps with BingoCube

**Primal**: Songbird (P2P Networking & Discovery)  
**Use Case**: Visual peer trust visualization  
**Duration**: 10 minutes

---

## Scenario

You're connecting to peers in a Songbird network. As you interact with each peer, trust grows. BingoCube provides a **visual trust indicator** that becomes more recognizable over time.

### Trust Evolution

**New Peer** (x=0.1):
- Just connected
- Minimal trust
- Almost empty BingoCube
- "Who is this?"

**Establishing** (x=0.4):
- Exchanged some data
- Building trust
- Pattern emerging
- "I'm starting to recognize you"

**Trusted** (x=0.8):
- Long history
- Reliable connection
- Full pattern visible
- "I know and trust you"

---

## Why This Works

### Trust Metrics → Visual Pattern

```rust
fn compute_trust_score(peer: &Peer) -> f64 {
    let uptime = peer.uptime.as_secs() as f64 / 86400.0; // Days online
    let reliability = peer.successful_requests as f64 
                    / peer.total_requests.max(1) as f64;
    let bandwidth = peer.shared_bandwidth / peer.expected_bandwidth;
    
    let trust = (uptime * 0.3 + reliability * 0.4 + bandwidth * 0.3);
    trust.clamp(0.0, 1.0)
}
```

As metrics improve → x increases → more cells reveal → pattern becomes recognizable.

### Visual Recognition
After seeing a peer's full BingoCube (x=1.0), you can recognize them at lower x levels:
- See x=0.5 pattern → "That's peer-songbird-42!"
- Instant visual verification
- No need to check hex IDs

---

## Demo

### Boards
```
Board A (Peer Identity):
  Seed: BLAKE3(peer_id || public_key)
  Fixed: Permanent peer identity
  
Board B (Connection History):
  Seed: BLAKE3(uptime || reliability || bandwidth || timestamp)
  Dynamic: Updates as metrics change
```

### Trust Evolution Timeline

**t=0min (New Connection, x=0.1)**:
```
┌───────────┐
│ · · · · ·│
│ · 🟦 · · ·│
│ · · ✱ · ·│
│ · · · · ·│
│ · · · · ·│
└───────────┘
Trust: 10% | "New peer, minimal trust"
```

**t=1hour (Exchanging Data, x=0.3)**:
```
┌───────────┐
│ · · 🟥 · ·│
│ · 🟦 · 🟨 ·│
│ 🟩 · ✱ · ·│
│ · · · 🟦 ·│
│ · · · · ·│
└───────────┘
Trust: 30% | "Pattern starting to emerge"
```

**t=1day (Reliable, x=0.7)**:
```
┌───────────┐
│ 🟦 🟩 🟥 🟨 ·│
│ 🟨 🟦 🟩 🟨 🟥│
│ 🟩 🟥 ✱ 🟦 🟨│
│ 🟦 🟩 🟨 🟦 🟩│
│ 🟥 🟨 🟩 · 🟦│
└───────────┘
Trust: 70% | "Recognizable, trusted"
```

**t=1week (Highly Trusted, x=0.95)**:
```
┌───────────┐
│ 🟦 🟩 🟥 🟨 🟦│
│ 🟨 🟦 🟩 🟨 🟥│
│ 🟩 🟥 ✱ 🟦 🟨│
│ 🟦 🟩 🟨 🟦 🟩│
│ 🟥 🟨 🟩 🟥 🟦│
└───────────┘
Trust: 95% | "Fully trusted peer"
```

---

## Federation Tower Trust

In Songbird's multi-tower federation, each tower has a BingoCube showing its trust level:

```
Tower Alpha (x=0.9)     Tower Beta (x=0.6)      Tower Gamma (x=0.2)
Highly Trusted          Moderate Trust          Low Trust

┌───────────┐          ┌───────────┐          ┌───────────┐
│ 🟦 🟩 🟥 🟨 🟦│          │ 🟥 🟨 · 🟩 🟦│          │ · · · 🟩 ·│
│ 🟨 🟦 🟩 🟨 🟥│          │ · 🟦 🟥 · 🟨│          │ · · · · ·│
│ 🟩 🟥 ✱ 🟦 🟨│          │ 🟩 · ✱ 🟦 ·│          │ · · ✱ 🟦 ·│
│ 🟦 🟩 🟨 🟦 🟩│          │ 🟦 🟨 · · 🟥│          │ · 🟨 · · ·│
│ 🟥 🟨 🟩 · 🟦│          │ · 🟩 🟦 🟨 ·│          │ · · 🟦 · ·│
└───────────┘          └───────────┘          └───────────┘

Route through          Consider              Avoid
```

**Federation Decision**: Route traffic through towers with x > 0.7.

---

## Trust Degradation

Trust can also **decrease** if metrics worsen:

```
t=0: x=0.8 (trusted)
     Peer goes offline for 48 hours
     
t=48h: x=0.4 (trust degraded)
       Pattern partially hidden
       "Where did you go?"
       
t=72h: Peer returns, metrics improve
       x grows back to 0.7
       "Welcome back, rebuilding trust"
```

The BingoCube **reflects current state**, not historical maximum.

---

## Code Example

```rust
use bingocube_core::{BingoCube, Config};

struct PeerTrust {
    peer_id: String,
    uptime: Duration,
    successful_requests: u64,
    total_requests: u64,
    shared_bandwidth: u64,
}

impl PeerTrust {
    fn compute_trust_score(&self) -> f64 {
        let uptime_factor = (self.uptime.as_secs() as f64 / 86400.0).min(1.0);
        let reliability = self.successful_requests as f64 
                        / self.total_requests.max(1) as f64;
        let trust = (uptime_factor * 0.4 + reliability * 0.6);
        trust.clamp(0.0, 1.0)
    }
    
    fn generate_bingocube(&self) -> Result<BingoCube> {
        // Board A: Peer ID (permanent)
        let board_a_seed = blake3::hash(self.peer_id.as_bytes());
        
        // Board B: Connection metrics (dynamic)
        let mut metrics = Vec::new();
        metrics.extend_from_slice(&self.uptime.as_secs().to_le_bytes());
        metrics.extend_from_slice(&self.successful_requests.to_le_bytes());
        metrics.extend_from_slice(&self.total_requests.to_le_bytes());
        let board_b_seed = blake3::hash(&metrics);
        
        // Composite seed
        let mut seed = Vec::new();
        seed.extend_from_slice(board_a_seed.as_bytes());
        seed.extend_from_slice(board_b_seed.as_bytes());
        
        BingoCube::from_seed(&seed, Config::default())
    }
    
    fn visualize(&self) -> Result<()> {
        let cube = self.generate_bingocube()?;
        let trust_score = self.compute_trust_score();
        let subcube = cube.subcube(trust_score)?;
        
        println!("Peer: {}", self.peer_id);
        println!("Trust: {:.0}% ({:.0}/25 cells)", 
                 trust_score * 100.0, 
                 subcube.revealed_count());
        
        // Render visual...
        Ok(())
    }
}
```

---

## Running the Demo

```bash
cd showcase/integration/02-songbird-trust
./demo.sh
```

The demo will:
1. Simulate peer connection
2. Show trust evolution over time (animated)
3. Display federation tower trust levels
4. Demonstrate trust degradation and recovery

---

## Multi-Modal Enhancement (Future)

### Audio Sonification
As trust grows, the peer's "sound" becomes richer:
- **x=0.1**: Single quiet beep
- **x=0.5**: Melodic pattern emerging
- **x=0.9**: Full harmonic chord

**Spatial audio**: Trusted peers = close/loud, untrusted = distant/quiet

### Animation
- **Flow particles**: Data flowing from trusted peers
- **Pulse effects**: Active connections pulse
- **Color saturation**: Higher trust = more vibrant colors

---

## Success Criteria

You should understand:
- ✅ How trust metrics map to x parameter
- ✅ Why visual patterns enable quick recognition
- ✅ How trust evolution is visible over time
- ✅ Why federation decisions become intuitive

---

**Next**: [03 - NestGate Content Fingerprints](../03-nestgate-content/)

