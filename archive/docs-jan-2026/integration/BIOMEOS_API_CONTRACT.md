# BiomeOS API Contract for PetalTongue

**Version:** 1.0  
**Date:** January 3, 2026  
**Status:** Draft (awaiting biomeOS implementation)

---

## 🎯 Purpose

This document specifies the HTTP REST API contract between petalTongue (client) and biomeOS (server) for primal discovery, topology visualization, and trust management.

---

## 🏗️ Architecture

```
┌──────────────┐     HTTP REST      ┌─────────────────┐
│ petalTongue  │ ←────────────────→ │  biomeOS API    │
│   (Client)   │   JSON over HTTP   │    (Server)     │
└──────────────┘                    └────────┬────────┘
                                             │
                                    ┌────────┴────────┐
                                    │                 │
                                    ▼                 ▼
                            ┌─────────────┐   ┌─────────────┐
                            │  Songbird   │   │  BearDog    │
                            │ (Discovery) │   │   (Trust)   │
                            └─────────────┘   └─────────────┘
                                 tarpc            HTTP
```

**Key Points:**
- petalTongue speaks HTTP REST only
- biomeOS API translates between HTTP (external) and tarpc/HTTP (internal)
- Clean protocol separation

---

## 📡 Base Configuration

### Endpoint Discovery

**Environment Variable:** `BIOMEOS_URL`

**Default:** `http://localhost:3000`

**Examples:**
```bash
# Local development
export BIOMEOS_URL=http://localhost:9000

# Production
export BIOMEOS_URL=https://biomeos.production:9000

# Remote testing
export BIOMEOS_URL=http://tower2:9000
```

### Request Format

**Protocol:** HTTP/1.1 or HTTP/2  
**Content-Type:** `application/json`  
**Timeout:** 10 seconds (client-side)

---

## 🔍 Phase 1: Core Discovery API

### 1.1 Health Check

**Purpose:** Verify biomeOS API is available

**Endpoint:** `GET /api/v1/health`

**Request:** None (GET)

**Response:**
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "timestamp": 1735862400
}
```

**Status Codes:**
- `200 OK` - API is healthy
- `503 Service Unavailable` - API is starting/degraded

**petalTongue Behavior:**
- Check on startup
- Retry every 5 seconds if unavailable
- Fall back to mock mode if consistently failing

---

### 1.2 Discover Primals

**Purpose:** Get list of all discovered primals with their capabilities

**Endpoint:** `GET /api/v1/primals`

**Request:** None (GET)

**Response:**
```json
{
  "primals": [
    {
      "id": "tower2",
      "name": "tower2",
      "primal_type": "tower",
      "endpoint": "http://tower2:8080",
      "capabilities": ["orchestration", "federation"],
      "health": "healthy",
      "last_seen": 1735862400
    },
    {
      "id": "local-songbird",
      "name": "Songbird on local",
      "primal_type": "songbird",
      "endpoint": "http://localhost:8080",
      "capabilities": ["discovery", "orchestration"],
      "health": "healthy",
      "last_seen": 1735862400
    }
  ]
}
```

**Field Specifications:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | ✅ Yes | Unique primal identifier |
| `name` | string | ✅ Yes | Human-readable name |
| `primal_type` | string | ✅ Yes | Display type (NOT for routing!) |
| `endpoint` | string | ✅ Yes | HTTP/tarpc/gRPC endpoint URL |
| `capabilities` | array[string] | ✅ Yes | Capability list (routing basis) |
| `health` | string | ✅ Yes | "healthy" \| "warning" \| "critical" \| "unknown" |
| `last_seen` | integer | ✅ Yes | Unix timestamp (seconds) |

**Status Codes:**
- `200 OK` - Success
- `500 Internal Server Error` - Discovery failed
- `503 Service Unavailable` - Songbird not available

**petalTongue Behavior:**
- Poll every 5 seconds (if auto-refresh enabled)
- Cache last successful response
- Fall back to cached data if request fails

---

## 🕸️ Phase 2: Topology API

### 2.1 Get Topology

**Purpose:** Get connections (edges) between primals

**Endpoint:** `GET /api/v1/topology`

**Request:** None (GET)

**Response:**
```json
[
  {
    "from": "local-songbird",
    "to": "tower2",
    "edge_type": "federation",
    "label": "tarpc connection"
  },
  {
    "from": "beardog-1",
    "to": "tower2",
    "edge_type": "trust_evaluation",
    "label": null
  }
]
```

**Field Specifications:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `from` | string | ✅ Yes | Source primal ID |
| `to` | string | ✅ Yes | Target primal ID |
| `edge_type` | string | ✅ Yes | Connection type |
| `label` | string \| null | ❌ No | Display label (optional) |

**Common Edge Types:**
- `federation` - Primal-to-primal federation
- `discovery` - Discovery relationship
- `trust_evaluation` - Trust verification
- `data_flow` - Data transfer
- `capability_invocation` - API call

**Status Codes:**
- `200 OK` - Success
- `500 Internal Server Error` - Failed to aggregate topology

**petalTongue Behavior:**
- Request alongside primal discovery
- If endpoint returns 404, infer topology from capabilities
- Update graph engine with edges

---

## 🔐 Phase 3: Trust Enrichment

### 3.1 Discover Primals with Trust

**Purpose:** Get primals with trust evaluation included

**Endpoint:** `GET /api/v1/primals?include_trust=true`

**Request:** Query parameter `include_trust=true`

**Response:**
```json
{
  "primals": [
    {
      "id": "tower2",
      "name": "tower2",
      "primal_type": "tower",
      "endpoint": "http://tower2:8080",
      "capabilities": ["orchestration", "federation"],
      "health": "healthy",
      "last_seen": 1735862400,
      "trust": {
        "trust_level": 1,
        "family_id": "iidn",
        "allowed_capabilities": ["orchestration"],
        "denied_capabilities": [],
        "last_evaluation": 1735862395
      }
    }
  ]
}
```

**Trust Field Specification:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `trust_level` | integer | ✅ Yes | 0=none, 1=limited, 2=elevated, 3=full |
| `family_id` | string \| null | ❌ No | Trust family identifier |
| `allowed_capabilities` | array[string] | ✅ Yes | Capabilities allowed at this trust level |
| `denied_capabilities` | array[string] | ✅ Yes | Capabilities denied |
| `last_evaluation` | integer | ✅ Yes | Unix timestamp of last trust eval |

**petalTongue Behavior:**
- If `trust` field present, display trust badges
- If absent, show as "Unknown Trust"
- Color-code by trust level

---

## 🎯 Phase 4: Trust Management

### 4.1 Elevate Trust

**Purpose:** Elevate trust level for a primal

**Endpoint:** `POST /api/v1/primals/{id}/elevate`

**Request:**
```json
{
  "new_trust_level": 2,
  "reason": "User approved via petalTongue UI",
  "authorized_by": "user@hostname"
}
```

**Response:**
```json
{
  "success": true,
  "primal_id": "tower2",
  "previous_trust_level": 1,
  "new_trust_level": 2,
  "timestamp": 1735862500
}
```

**Status Codes:**
- `200 OK` - Trust elevated successfully
- `400 Bad Request` - Invalid trust level
- `403 Forbidden` - Not authorized to elevate
- `404 Not Found` - Primal not found

**petalTongue Behavior:**
- Show confirmation dialog before elevating
- Display trust change in UI
- Refresh primal list after elevation

---

## 🚨 Error Handling

### Standard Error Response

**Format:**
```json
{
  "error": {
    "code": "DISCOVERY_FAILED",
    "message": "Failed to discover primals from Songbird",
    "details": "Connection refused: http://localhost:8080",
    "timestamp": 1735862400
  }
}
```

**Common Error Codes:**

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `DISCOVERY_FAILED` | 500 | Failed to query Songbird |
| `TRUST_EVALUATION_FAILED` | 500 | Failed to query BearDog |
| `INVALID_TRUST_LEVEL` | 400 | Invalid trust level value |
| `PRIMAL_NOT_FOUND` | 404 | Primal ID not found |
| `UNAUTHORIZED` | 403 | Not authorized for operation |
| `SERVICE_UNAVAILABLE` | 503 | Dependency not available |

### petalTongue Error Handling

**Strategy:**
1. Try request with 10-second timeout
2. If fails, log warning with details
3. Fall back to cached data (if available)
4. Fall back to mock data (if no cache)
5. Display warning banner in UI
6. Retry on next refresh interval

---

## 🔄 Versioning

### URL Versioning

**Current:** `/api/v1/`  
**Future:** `/api/v2/`

**Breaking Changes:**
- New major version (v1 → v2)
- Old version supported for 6 months

**Non-Breaking Changes:**
- Add optional fields (backward compatible)
- Add new endpoints (no version change)

### Capability Detection

**petalTongue checks:**
```rust
// Check if trust enrichment is available
let supports_trust = self.health_check()
    .await?
    .capabilities
    .contains("trust_enrichment");
```

---

## 🧪 Testing Contract

### Mock Mode

**For petalTongue development:**
```bash
export PETALTONGUE_MOCK_MODE=true
```

**Returns hardcoded data without any network calls.**

### Integration Testing

**Test Sequence:**

1. **Health Check**
   ```bash
   curl http://localhost:9000/api/v1/health
   ```

2. **Discovery**
   ```bash
   curl http://localhost:9000/api/v1/primals
   ```

3. **Topology**
   ```bash
   curl http://localhost:9000/api/v1/topology
   ```

4. **Trust Enrichment**
   ```bash
   curl http://localhost:9000/api/v1/primals?include_trust=true
   ```

5. **Trust Elevation**
   ```bash
   curl -X POST http://localhost:9000/api/v1/primals/tower2/elevate \
     -H "Content-Type: application/json" \
     -d '{"new_trust_level": 2, "reason": "test"}'
   ```

---

## 📊 Implementation Checklist

### biomeOS Team

**Phase 1: Core API**
- [ ] Create `biomeos-api` crate
- [ ] Implement axum server
- [ ] Add `/api/v1/health` endpoint
- [ ] Add `/api/v1/primals` endpoint
- [ ] Integrate with Songbird (tarpc)
- [ ] Test with petalTongue

**Phase 2: Topology**
- [ ] Add `/api/v1/topology` endpoint
- [ ] Aggregate Songbird connections
- [ ] Test with petalTongue topology view

**Phase 3: Trust**
- [ ] Add trust enrichment to `/api/v1/primals`
- [ ] Integrate with BearDog
- [ ] Test with petalTongue trust badges

**Phase 4: Elevation**
- [ ] Add `/api/v1/primals/{id}/elevate` endpoint
- [ ] Wire to BearDog elevation API
- [ ] Test with petalTongue elevation UI

### petalTongue Team

**Improvements**
- [x] Document API contract (this file)
- [ ] Add health check support
- [ ] Improve error messages
- [ ] Add connection status UI
- [ ] Add retry logic
- [ ] Add trust badge UI
- [ ] Add elevation UI

---

## 🎊 Success Criteria

### Phase 1 Success

**Given:** biomeOS API running on port 9000  
**When:** petalTongue starts with `BIOMEOS_URL=http://localhost:9000`  
**Then:**
- ✅ Health check succeeds
- ✅ Primal discovery returns real data
- ✅ UI shows discovered primals
- ✅ No fallback to mock mode

### Full Integration Success

**Given:** All phases implemented  
**When:** User interacts with petalTongue  
**Then:**
- ✅ Real-time primal discovery
- ✅ Topology visualization with real connections
- ✅ Trust levels displayed
- ✅ Trust elevation works
- ✅ No hardcoded primal knowledge
- ✅ Pure capability-based routing

---

## 📚 References

**petalTongue:**
- `crates/petal-tongue-api/src/biomeos_client.rs` - Client implementation
- `crates/petal-tongue-core/src/config.rs` - Configuration
- `ENV_VARS.md` - Environment variables

**biomeOS:**
- `PETALTONGUE_LIVE_INTEGRATION_GAPS.md` - Gap discovery
- `crates/biomeos-core/src/primal_client/` - Universal Primal Client

---

**Contract Version:** 1.0  
**Status:** Draft (awaiting biomeOS implementation)  
**Last Updated:** January 3, 2026

---

*This contract enables capability-based integration between petalTongue and biomeOS.*

**No hardcoded primal names. Pure capability discovery.** 🌸🚀

