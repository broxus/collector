## collector

### How to build

```bash
git clone https://github.com/broxus/collector.git
cd collector
cargo build --release
target/release/collector --help
```

### How to use

```bash
# 1. Generate deposit address
collector addr <secret key in hex format> 

# 2. Send TON to this address

# 3. Collect it somewhere
collector msg <secret key in hex format> --init --to <target address in raw format> | base64 -d > msg.boc
tonos-cli --url https://main.ton.dev sendfile msg.boc
```
