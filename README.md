# Pixel Archives Backend

A collaborative pixel art platform backend built with Rust, Axum, and Solana.

## Features

- **Collaborative Canvas**: 32x32 pixel canvases with real-time updates via WebSocket
- **Pixel Ownership**: Claim pixels on published canvases using Solana payments
- **NFT Minting**: Convert completed canvases into Metaplex-compatible NFTs
- **Creator Royalties**: Automatic royalty distribution based on pixel ownership

## Tech Stack

- **Framework**: Axum (async Rust web framework)
- **Database**: PostgreSQL with SeaORM
- **Cache**: Redis (rate limiting, pixel locks, blockhash cache)
- **Blockchain**: Solana (devnet/mainnet)
- **Real-time**: WebSocket for live canvas updates

## Quick Start

### Prerequisites

- Rust 1.75+
- PostgreSQL 14+
- Redis 7+
- Solana CLI (optional, for testing)

### Setup

1. Clone and configure:
```bash
cp .env.example .env
# Edit .env with your database and Redis URLs
```

2. Run migrations:
```bash
cargo run
```

3. The server starts on `http://localhost:8080`

## Environment Variables

See [.env.example](.env.example) for all configuration options.

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | PostgreSQL connection string |
| `REDIS_URL` | Redis connection string |
| `JWT_SECRET` | Secret key for JWT signing |
| `SOLANA_RPC_URL` | Solana RPC endpoint |
| `SOLANA_PROGRAM_ID` | Deployed program address |

## API Documentation

See [docs/API.md](docs/API.md) for complete API reference including:

- JSON-RPC method reference
- WebSocket events
- Error codes
- Rate limits

## Project Structure

```
src/
├── api/                # HTTP handlers and routing
│   ├── dispatcher.rs   # JSON-RPC method dispatch
│   ├── methods/        # Method implementations
│   ├── router.rs       # HTTP router and cookie handling
│   └── types/          # Request/response types
├── config/             # Configuration management
├── error/              # Error types and JSON-RPC formatting
├── infrastructure/     # Infrastructure components
│   ├── cache/          # Redis and local caching
│   └── db/             # Database entities and repositories
├── middleware/         # Rate limiting and logging
├── services/           # Business logic
│   ├── auth/           # JWT and wallet authentication
│   ├── canvas/         # Canvas lifecycle management
│   ├── nft/            # NFT metadata and minting
│   ├── pixel/          # Pixel placement and bidding
│   └── solana/         # Solana RPC client
├── utils/              # Server utilities
└── ws/                 # WebSocket handler and room management
```

## Development

```bash
# Run with hot reload
cargo watch -x run

# Run tests
cargo test

# Lint
cargo clippy

# Format
cargo fmt
```

## License

Source Available License - See [LICENSE](LICENSE) for details.

**You may** view and study this code for educational purposes.  
**You may not** use this code in commercial or production environments without explicit permission.
