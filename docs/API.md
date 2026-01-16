# Pixel Archives API Documentation

## Overview

Pixel Archives is a collaborative pixel art platform where users create, claim, and mint 32x32 pixel canvases as NFTs on Solana.

**Protocol:** JSON-RPC 2.0 over HTTP POST

**Authentication:** Cookie-based JWT tokens (automatically set on login)

---

## Getting Started

### 1. Connect Wallet

Users authenticate by signing a message with their Solana wallet.

```json
{
  "jsonrpc": "2.0",
  "method": "auth.register",
  "params": {
    "wallet": "AhVo77xR2QTxBQujDUCVMPcvGRXw8z7JiXhywDoVF5Ud",
    "message": "Sign in to Pixel Archives: 1705420800000",
    "signature": "base58_encoded_signature",
    "username": "artist123"
  },
  "id": 1
}
```

### 2. Create a Canvas

```json
{
  "jsonrpc": "2.0",
  "method": "canvas.create",
  "params": {
    "name": "My First Canvas"
  },
  "id": 2
}
```

### 3. Place a Pixel

```json
{
  "jsonrpc": "2.0",
  "method": "pixel.place",
  "params": {
    "canvas_id": "550e8400-e29b-41d4-a716-446655440000",
    "x": 15,
    "y": 15,
    "color": 23
  },
  "id": 3
}
```

---

## Authentication

All authenticated endpoints require a valid JWT token. Tokens are automatically set as HTTP-only cookies on successful login/register.

### Methods

| Method | Description | Auth Required |
|--------|-------------|---------------|
| `auth.register` | Create new account with wallet signature | No |
| `auth.login` | Authenticate existing account | No |
| `auth.logout` | Invalidate current session | Yes |
| `auth.refresh` | Refresh access token using refresh token | Yes |

---

## JSON-RPC Reference

### Request Format

```json
{
  "jsonrpc": "2.0",
  "method": "namespace.action",
  "params": { ... },
  "id": 1
}
```

### Response Format

**Success:**
```json
{
  "jsonrpc": "2.0",
  "result": { ... },
  "id": 1
}
```

**Error:**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Invalid params: canvas_id is required"
  },
  "id": 1
}
```

---

## Auth Methods

### auth.register

Create a new user account.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `wallet` | string | Yes | Solana wallet address (base58) |
| `message` | string | Yes | Message that was signed |
| `signature` | string | Yes | Base58-encoded signature |
| `username` | string | No | Display name (3-20 chars, alphanumeric) |

**Response:**
```json
{
  "user": {
    "id": "uuid",
    "wallet_address": "AdVo76x...",
    "username": "artist123"
  }
}
```

**Errors:** `-32010` User exists, `-32013` Username exists, `-32012` Invalid signature

---

### auth.login

Authenticate an existing user.

**Parameters:** Same as `auth.register` (username is ignored)

**Response:** Same as `auth.register`

**Errors:** `-32011` User not found, `-32012` Invalid signature

---

### auth.logout

End the current session.

**Parameters:** None (uses cookie)

**Response:**
```json
{
  "success": true
}
```

---

### auth.refresh

Refresh the access token.

**Parameters:** None (uses refresh_token cookie)

**Response:**
```json
{
  "user": {
    "id": "uuid",
    "wallet_address": "AhVo77x...",
    "username": "artist123"
  }
}
```

**Errors:** `-32021` Token expired

---

## Canvas Methods

### canvas.create

Create a new 32x32 canvas.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | Yes | Canvas name (unique per user) |
| `initial_color` | integer | No | Default color index (0-63), default: 10 (white) |

**Response:**
```json
{
  "id": "uuid",
  "name": "My Canvas",
  "invite_code": "ABC123",
  "state": "draft",
  "owner_id": "uuid",
  "canvas_pda": null,
  "mint_address": null
}
```

**Errors:** `-32037` Canvas name exists

---

### canvas.list

List all canvases the user owns or collaborates on.

**Parameters:** None

**Response:**
```json
{
  "owned": [
    { "id": "uuid", "name": "Canvas 1", ... }
  ],
  "collaborating": [
    { "id": "uuid", "name": "Shared Canvas", ... }
  ]
}
```

---

### canvas.get

Get canvas details including all pixel data.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `canvas_id` | uuid | Yes | Canvas UUID |

**Response:**
```json
{
  "canvas": { ... },
  "pixel_colors": "base64_encoded_768_bytes",
  "owned_pixels": [
    { "x": 15, "y": 15, "owner_id": "uuid", "price_lamports": 1000000 }
  ]
}
```

The `pixel_colors` field is a base64-encoded byte array where each 3 bytes encode 4 pixel colors (6-bit packed format).

---

### canvas.join

Join a canvas as a collaborator using an invite code.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `invite_code` | string | Yes | 6-character invite code |

**Response:**
```json
{
  "success": true,
  "canvas_id": "uuid"
}
```

---

### canvas.publish

Initiate publishing the canvas to Solana blockchain.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `canvas_id` | uuid | Yes | Canvas UUID |

**Response:**
```json
{
  "success": true,
  "state": "publishing",
  "pixel_colors_packed": "base64_encoded_768_bytes"
}
```

---

### canvas.confirmPublish

Confirm the on-chain publish transaction.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `canvas_id` | uuid | Yes | Canvas UUID |
| `signature` | string | Yes | Solana transaction signature |
| `canvas_pda` | string | Yes | Canvas PDA address |

**Response:**
```json
{
  "success": true,
  "state": "published",
  "canvas_pda": "CanvasPDA..."
}
```

---

### canvas.cancelPublish

Cancel a pending publish operation.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `canvas_id` | uuid | Yes | Canvas UUID |

**Response:**
```json
{
  "success": true,
  "state": "draft"
}
```

---

### canvas.delete

Delete a draft canvas.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `canvas_id` | uuid | Yes | Canvas UUID |

**Response:**
```json
{
  "success": true
}
```

**Errors:** `-32031` Invalid canvas state transition (cannot delete published canvas)

---

## Pixel Methods

### pixel.place

Place a bid on a pixel. If the pixel is owned by the caller, no payment is required.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `canvas_id` | uuid | Yes | Canvas UUID |
| `x` | integer | Yes | X coordinate (0-31) |
| `y` | integer | Yes | Y coordinate (0-31) |
| `color` | integer | Yes | Color index (0-63) |
| `bid_lamports` | integer | No | Bid amount in lamports (required if outbidding) |

**Response:**
```json
{
  "success": true,
  "x": 15,
  "y": 15,
  "color": 23,
  "requires_confirmation": true,
  "previous_owner_wallet": "PrevOwner..."
}
```

When `requires_confirmation` is `true`, you must submit a Solana transaction and call `pixel.confirm`.

---

### pixel.confirm

Confirm a pixel placement after on-chain payment.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `canvas_id` | uuid | Yes | Canvas UUID |
| `x` | integer | Yes | X coordinate |
| `y` | integer | Yes | Y coordinate |
| `color` | integer | Yes | Color index |
| `signature` | string | Yes | Solana transaction signature |

**Response:**
```json
{
  "success": true,
  "x": 15,
  "y": 15,
  "color": 23,
  "owner_id": "uuid",
  "price_lamports": 1000000
}
```

---

### pixel.cancel

Cancel a pending pixel bid (release the lock).

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `canvas_id` | uuid | Yes | Canvas UUID |
| `x` | integer | Yes | X coordinate |
| `y` | integer | Yes | Y coordinate |

**Response:**
```json
{
  "success": true
}
```

---

### pixel.paint

Paint a pixel you already own (free color change).

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `canvas_id` | uuid | Yes | Canvas UUID |
| `x` | integer | Yes | X coordinate |
| `y` | integer | Yes | Y coordinate |
| `color` | integer | Yes | New color index |
| `signature` | string | Yes | Wallet signature for proof |

**Response:**
```json
{
  "success": true,
  "x": 15,
  "y": 15,
  "color": 42
}
```

---

## NFT Methods

### nft.announceMint

Announce minting countdown to all collaborators. Locks the canvas for 30 seconds.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `canvas_id` | uuid | Yes | Canvas UUID |

**Response:**
```json
{
  "success": true,
  "state": "mint_pending",
  "countdown_seconds": 30
}
```

---

### nft.cancelMintCountdown

Cancel the minting countdown.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `canvas_id` | uuid | Yes | Canvas UUID |

**Response:**
```json
{
  "success": true
}
```

---

### nft.prepareMetadata

Generate NFT metadata and image. Returns URIs for use in the mint transaction.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `canvas_id` | uuid | Yes | Canvas UUID |

**Response:**
```json
{
  "success": true,
  "metadata_uri": "https://api.../nft/{id}/metadata.json",
  "image_uri": "data:image/png;base64,...",
  "image_gateway_url": "data:image/png;base64,...",
  "metadata_gateway_url": "",
  "creators": [
    { "address": "OwnerWallet...", "share": 50 },
    { "address": "Contributor1...", "share": 30 },
    { "address": "Contributor2...", "share": 20 }
  ]
}
```

Creator shares are calculated based on pixel ownership value.

---

### nft.mint

Initiate the NFT minting process.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `canvas_id` | uuid | Yes | Canvas UUID |

**Response:**
```json
{
  "success": true,
  "state": "minting"
}
```

---

### nft.confirmMint

Confirm the NFT mint after on-chain transaction.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `canvas_id` | uuid | Yes | Canvas UUID |
| `signature` | string | Yes | Solana transaction signature |
| `mint_address` | string | Yes | NFT mint address |

**Response:**
```json
{
  "success": true,
  "state": "minted"
}
```

---

### nft.cancelMint

Cancel a pending mint operation.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `canvas_id` | uuid | Yes | Canvas UUID |

**Response:**
```json
{
  "success": true,
  "state": "published"
}
```

---

## WebSocket API

Real-time updates are delivered via WebSocket connections.

**Endpoint:** `wss://api.pixelarchives.io/ws/{canvas_id}`

### Connection

Include the access token cookie when connecting. The server will authenticate and associate the connection with the user.

### Server Messages

**Pixel Update:**
```json
{
  "type": "pixel_update",
  "payload": {
    "x": 15,
    "y": 15,
    "color": 23,
    "owner_id": "uuid",
    "price_lamports": 1000000
  }
}
```

**Mint Countdown:**
```json
{
  "type": "mint_countdown",
  "payload": {
    "seconds": 30
  }
}
```

**Mint Countdown Cancelled:**
```json
{
  "type": "mint_countdown_cancelled"
}
```

**Minting Started:**
```json
{
  "type": "minting_started"
}
```

**Minted:**
```json
{
  "type": "minted",
  "payload": {
    "mint_address": "NFTMint..."
  }
}
```

**Minting Failed:**
```json
{
  "type": "minting_failed",
  "payload": {
    "reason": "Cancelled by user"
  }
}
```

---

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| `-32602` | Invalid Params | Missing or invalid request parameters |
| `-32601` | Method Not Found | Unknown RPC method |
| `-32603` | Internal Error | Server-side error |
| `-32010` | User Exists | Wallet already registered |
| `-32011` | User Not Found | Wallet not registered |
| `-32012` | Invalid Signature | Wallet signature verification failed |
| `-32013` | Username Exists | Username already taken |
| `-32020` | Unauthorized | Authentication required |
| `-32021` | Token Expired | JWT token has expired |
| `-32030` | Canvas Not Found | Canvas UUID does not exist |
| `-32031` | Invalid State Transition | Operation not allowed in current canvas state |
| `-32034` | Not Canvas Owner | Only the canvas owner can perform this action |
| `-32035` | Not Collaborator | User is not a collaborator on this canvas |
| `-32037` | Canvas Name Exists | A canvas with this name already exists |
| `-32040` | Pixel Locked | Pixel is being edited by another user |
| `-32041` | Bid Too Low | Bid must exceed current pixel price |
| `-32042` | Cooldown Active | Must wait before placing another pixel |
| `-32060` | Transaction Failed | Solana transaction verification failed |
| `-32061` | Solana RPC Error | Solana network communication error |
| `-32070` | Database Error | Database temporarily unavailable |
| `-32071` | Redis Error | Cache temporarily unavailable |
| `-32072` | Serialization Error | Failed to serialize/deserialize data |
| `-32081` | Rate Limit Exceeded | Too many requests, try again later |

---

## Rate Limits

Rate limits are applied per-user (by JWT) or per-IP (for unauthenticated requests).

| Category | Limit | Window |
|----------|-------|--------|
| Auth (login/register) | 10 | 60s |
| Pixel operations | 30 | 60s |
| Canvas operations | 5 | 60s |
| Solana operations | 20 | 60s |

When rate limited, you receive a `-32081` error with `Retry-After` header.

---

## Color Palette

The canvas supports 64 colors (indices 0-63):

| Range | Description |
|-------|-------------|
| 0-10 | Grayscale (black to white) |
| 11-15 | Reds |
| 16-23 | Oranges to Greens |
| 24-31 | Greens to Cyans |
| 32-39 | Blues to Purples |
| 40-47 | Purples to Pinks |
| 48-55 | Browns and Earth tones |
| 56-63 | Pastels |

See `src/services/nft/image.rs` for exact RGB values.

---

## NFT Metadata Endpoints

### GET /nft/{canvas_id}/metadata.json

Returns Metaplex-compatible JSON metadata for minted NFTs.

### GET /nft/{canvas_id}/image.png

Returns the 512x512 PNG image of the canvas (16x upscaled from 32x32).
