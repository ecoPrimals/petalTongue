# BearDog Identity Verification with BingoCube

**Primal**: BearDog (Security & Identity)  
**Use Case**: Progressive trust identity protocol  
**Duration**: 10 minutes

---

## Scenario

Alice wants to prove her identity to Bob. Instead of showing a full credential immediately, she uses BingoCube for **progressive trust building**.

### The Protocol

1. **Initial Contact** (x=0.2):
   - Alice: "I am alice@ecoprimals.bio"
   - Alice generates BingoCube from her identity
   - Alice reveals 20% (5 cells)
   - Bob sees partial pattern → "Probably Alice"

2. **Challenge** (x=0.5):
   - Bob: "Prove it with challenge 'abc123'"
   - Alice generates new BingoCube with challenge
   - Alice reveals 50% (13 cells)
   - Bob verifies cells from x=0.2 are present → "Definitely Alice"

3. **Full Verification** (x=1.0):
   - Bob requests full reveal
   - Alice reveals all 25 cells
   - Bob stores pattern for future recognition
   - Trust established

---

## Why This Works

### Security Properties
- **Forgery**: ~2^(-32) at x=0.3, ~2^(-50) at x=0.5
- **Progressive**: Can't skip levels (nested masks)
- **Fresh**: Each challenge produces new Board B
- **Memorable**: Visual pattern easier than hex hash

### User Experience
- **Incremental**: Trust builds gradually
- **Visual**: Recognizable pattern emerges
- **Interactive**: Bob controls reveal level
- **Reversible**: Can restart if suspicious

---

## Demo

### Boards
```
Board A (Identity):
  Seed: BLAKE3(alice_seed || pub_key)
  Fixed: Never changes for Alice
  
Board B (Challenge):
  Seed: BLAKE3(timestamp || challenge)
  Dynamic: Changes per verification
```

### Progressive Reveals

**x=0.2 (Initial - 5 cells)**:
```
┌───────────┐
│ · · 🟥 · ·│
│ · 🟦 · · ·│
│ · · ✱ 🟨 ·│
│ 🟩 · · · ·│
│ · · · · ·│
└───────────┘
"I might be Alice"
```

**x=0.5 (Challenge - 13 cells)**:
```
┌───────────┐
│ 🟦 · 🟥 🟨 ·│
│ · 🟦 🟩 · 🟥│
│ 🟨 · ✱ 🟨 🟦│
│ 🟩 🟥 · 🟦 ·│
│ · 🟨 🟩 · ·│
└───────────┘
"I am definitely Alice"
```

**x=1.0 (Full - 25 cells)**:
```
┌───────────┐
│ 🟦 🟩 🟥 🟨 🟦│
│ 🟨 🟦 🟩 🟨 🟥│
│ 🟩 🟥 ✱ 🟦 🟨│
│ 🟦 🟩 🟨 🟦 🟩│
│ 🟥 🟨 🟩 🟥 🟦│
└───────────┘
"Full identity commitment"
```

---

## Multi-Factor Authentication

BingoCube supports multi-factor by showing **4 cubes simultaneously**:

```
WHO (identity)      WHAT (password)
┌───────────┐      ┌───────────┐
│ 🟦 🟩 🟥 🟨 🟦│      │ 🟥 🟨 🟦 🟩 🟥│
│ 🟨 🟦 🟩 🟨 🟥│      │ 🟦 🟩 🟨 🟦 🟨│
│ 🟩 🟥 ✱ 🟦 🟨│      │ 🟩 🟦 ✱ 🟥 🟩│
│ 🟦 🟩 🟨 🟦 🟩│      │ 🟨 🟥 🟩 🟨 🟦│
│ 🟥 🟨 🟩 🟥 🟦│      │ 🟦 🟨 🟥 🟩 🟥│
└───────────┘      └───────────┘

WHERE (device)      WHEN (time token)
┌───────────┐      ┌───────────┐
│ 🟩 🟦 🟥 🟨 🟩│      │ 🟨 🟩 🟦 🟥 🟨│
│ 🟨 🟥 🟦 🟩 🟨│      │ 🟥 🟦 🟩 🟨 🟥│
│ 🟦 🟩 ✱ 🟥 🟦│      │ 🟦 🟨 ✱ 🟩 🟦│
│ 🟥 🟨 🟩 🟦 🟥│      │ 🟩 🟥 🟨 🟦 🟩│
│ 🟦 🟩 🟨 🟥 🟦│      │ 🟨 🟦 🟥 🟩 🟨│
└───────────┘      └───────────┘
```

All 4 must match at the requested x level for full verification.

---

## Code Example

```rust
use bingocube_core::{BingoCube, Config};

// Alice generates identity BingoCube
let identity_seed = blake3::hash(b"alice_identity_master_seed");
let challenge = b"challenge_from_bob";
let timestamp = Instant::now();

// Board A: From identity (permanent)
let board_a_seed = blake3::hash(&[identity_seed.as_bytes(), b"_IDENTITY"].concat());

// Board B: From challenge + timestamp (ephemeral)
let mut board_b_input = Vec::new();
board_b_input.extend_from_slice(challenge);
board_b_input.extend_from_slice(&timestamp.elapsed().as_secs().to_le_bytes());
let board_b_seed = blake3::hash(&board_b_input);

// Generate composite seed
let mut composite_seed = Vec::new();
composite_seed.extend_from_slice(board_a_seed.as_bytes());
composite_seed.extend_from_slice(board_b_seed.as_bytes());

// Generate BingoCube
let cube = BingoCube::from_seed(&composite_seed, Config::default())?;

// Progressive reveal
let reveal_20 = cube.subcube(0.2)?; // Initial
let reveal_50 = cube.subcube(0.5)?; // Challenge
let reveal_100 = cube.subcube(1.0)?; // Full

// Bob verifies nested property
assert!(cube.verify_subcube(&reveal_20, 0.2));
assert!(cube.verify_subcube(&reveal_50, 0.5));

// Verify that reveal_20 ⊂ reveal_50 ⊂ reveal_100
for (pos, _) in &reveal_20.revealed {
    assert!(reveal_50.revealed.contains_key(pos));
    assert!(reveal_100.revealed.contains_key(pos));
}
```

---

## Running the Demo

```bash
cd showcase/integration/01-beardog-identity
./demo.sh
```

The demo will:
1. Generate Alice's identity BingoCube
2. Show progressive reveals (x=0.2, 0.5, 1.0)
3. Demonstrate challenge-response
4. Show multi-factor variant

---

## Success Criteria

You should understand:
- ✅ How progressive reveal builds trust
- ✅ Why nested masks prevent skipping
- ✅ How challenges keep verification fresh
- ✅ Why visual patterns are memorable

---

**Next**: [02 - Songbird Trust Stamps](../02-songbird-trust/)

