# Cosync Wire Protocol

## Transport

All data is sent over **QUIC** (RFC 9000) using bidirectional streams. Each message occupies one stream: the sender opens a stream, writes the framed message, and closes the write side. The receiver reads until EOF.

### Framing

```
┌─────────────────────────────────────────────────────────┐
│  length : u32 (big-endian)  │  payload : protobuf bytes │
└─────────────────────────────────────────────────────────┘
```

Every message is a length-prefixed `Envelope` protobuf. The length is a 4-byte big-endian unsigned integer. Maximum payload: 64 MiB (file chunks are typically 64 KiB).

---

## Envelope Structure

Defined in `proto/cosync.proto`:

```protobuf
message Envelope {
  bytes    sender_device_id = 1;   // SHA-256 of sender's Ed25519 public key
  bytes    hlc_timestamp    = 2;   // JSON-encoded HlcTimestamp (wall + logical + node)
  oneof payload {
    ClipboardUpdate   clipboard_update   = 10;
    FileMeta          file_meta          = 11;
    FileChunk         file_chunk         = 12;
    FileAck           file_ack           = 13;
    NotificationEvent notification_event = 14;
    Heartbeat         heartbeat          = 15;
    PairingRequest    pairing_request    = 16;
    PairingAck        pairing_ack        = 17;
  }
}
```

### Fields

| Field | Type | Description |
|---|---|---|
| `sender_device_id` | bytes | 32-byte SHA-256 fingerprint of the sender |
| `hlc_timestamp` | bytes | JSON-encoded `HlcTimestamp` — used for causal ordering and clock-skew detection |
| `payload` | oneof | One of the message variants below |

---

## Message Types

### ClipboardUpdate (tag 10)

Sent when the user explicitly shares clipboard content (or the clipboard monitor fires).

```protobuf
message ClipboardUpdate {
  bytes  content      = 1;   // raw bytes
  string content_type = 2;   // MIME type, e.g. "text/plain" or "image/png"
}
```

### FileMeta (tag 11)

Sent before file chunks; provides metadata so the receiver can prepare storage.

```protobuf
message FileMeta {
  string file_name    = 1;
  uint64 file_size    = 2;   // bytes
  string sha256       = 3;   // hex-encoded SHA-256 of full file
  string content_type = 4;   // MIME type
  uint32 total_chunks = 5;
  string transfer_id  = 6;   // stable ID for correlating chunks + ack
}
```

### FileChunk (tag 12)

One fixed-size slice of the file payload.

```protobuf
message FileChunk {
  string transfer_id  = 1;
  uint32 chunk_index  = 2;
  bytes  data         = 3;
}
```

Default chunk size: **64 KiB**. Chunks are sent sequentially with a 1 ms yield between each to avoid saturating the connection.

### FileAck (tag 13)

Sent by the original sender after all chunks are transmitted.

```protobuf
message FileAck {
  string transfer_id = 1;
  bool   success     = 2;
  string reason      = 3;   // non-empty on failure
}
```

### NotificationEvent (tag 14)

Mirrors a mobile OS notification. Sent by a companion Android/iOS app (M9, planned).

```protobuf
message NotificationEvent {
  string package_name = 1;
  string title        = 2;
  string text         = 3;
}
```

### Heartbeat (tag 15)

No payload. Sent periodically to keep the QUIC connection alive and detect peer loss.

```protobuf
message Heartbeat {}
```

### PairingRequest (tag 16)

Initial pairing handshake — sent by the initiating device.

```protobuf
message PairingRequest {
  string device_name            = 1;
  string public_key_fingerprint = 2;   // SHA-256 hex
  uint32 listening_port         = 3;
  bytes  token                  = 4;   // 32 random bytes
}
```

### PairingAck (tag 17)

Response from the accepting device.

```protobuf
message PairingAck {
  bool   accepted               = 1;
  string device_name            = 2;
  string public_key_fingerprint = 3;
  string reason                 = 4;   // non-empty if rejected
}
```

---

## Hybrid Logical Clock (HLC)

Every envelope carries an HLC timestamp encoded as JSON bytes in `hlc_timestamp`.

```json
{ "wall": 1721000000000, "logical": 3, "node": "a1b2c3..." }
```

| Field | Type | Description |
|---|---|---|
| `wall` | u64 | Unix milliseconds from the local system clock |
| `logical` | u16 | Logical counter — incremented when wall time hasn't advanced |
| `node` | string | Device fingerprint (last 8 hex chars) — breaks ties |

**Receive rule:** `max(local_wall, remote_wall)` → advance logical counter if needed. Envelopes with wall skew > 30 seconds are dropped.

HLC provides a total causal order across devices without requiring clock synchronisation, so clipboard and file events can be deduplicated and replayed in a consistent order.

---

## Pairing Handshake Sequence

```
Device A (initiator)               Device B (responder)
        │                                   │
        │── QUIC connect ──────────────────►│
        │                                   │
        │   PinnedCertVerifier validates     │
        │   B's fingerprint (from mDNS)     │
        │                                   │
        │── PairingRequest ────────────────►│
        │   {device_name, fingerprint,       │
        │    port, random_token}             │
        │                                   │
        │                    user approves  │
        │                                   │
        │◄── PairingAck ───────────────────│
        │   {accepted: true,                │
        │    device_name, fingerprint}      │
        │                                   │
        │  Both sides:                       │
        │  • upsert_device() in SQLite       │
        │  • add_fingerprint() to verifier   │
        │  • store QUIC connection handle    │
```

After pairing, future connections are automatically trusted via the `PinnedCertVerifier` — no further user interaction needed.

---

## TLS Configuration

| Setting | Value |
|---|---|
| TLS version | 1.3 only |
| Certificate type | Self-signed X.509 (Ed25519) via rcgen |
| Verification | Mutual — both sides verify the remote fingerprint against the pinned set |
| CA | None — trust is established by explicit pairing, not a certificate hierarchy |

The `PinnedCertVerifier` (in `transport.rs`) implements `rustls::server::ClientCertVerifier` and `rustls::client::ServerCertVerifier`. It rejects any certificate whose public-key SHA-256 is not in the in-memory pinned set (populated from SQLite on startup and updated on each successful pairing).
