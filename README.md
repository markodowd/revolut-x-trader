# revolut-trading

A Revolut trading bot for LTC/USD written in Rust.

## Building

### Local (x86_64)

```bash
cargo build --release
```

### Raspberry Pi Zero 2W (aarch64)

Install the target and cross-linker once:

```bash
rustup target add aarch64-unknown-linux-gnu
sudo apt install gcc-aarch64-linux-gnu
```

Then build:

```bash
cargo build --release --target aarch64-unknown-linux-gnu
```

The binary will be at `target/aarch64-unknown-linux-gnu/release/revolut-trading`.

## Deploying to the Pi

```bash
scp target/aarch64-unknown-linux-gnu/release/revolut-trading modowd@pi-revolut.local:~/
scp -r keys .env modowd@pi-revolut.local:~/
```

On the Pi:

```bash
chmod +x revolut-trading
./revolut-trading
```

## Configuration

Copy `.env.example` to `.env` and fill in your Revolut API credentials. Place your Ed25519 keys in the `keys/` directory:

- `keys/private.pem`
- `keys/public.pem`
