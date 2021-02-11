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
# 0. Generate secret key
secret=$(openssl genpkey -algorithm ED25519 | openssl asn1parse -offset 14 | cut -d ':' -f 4)

# 1. Generate deposit address
deposit_addr=$(collector addr "$secret") 

# 1.1. Set target address
target="0:B00BA005540C10B37E724470A4CAEF42420535567895693E6FCCF9FACF7B7012" 

# 2.1. Simple use case when all funds are collected and contract is destroyed after this transaction

# 2.1.1. Send TON to `$deposit_addr`

# 2.1.2. Collect it somewhere
collector msg "$secret"  \
  --init \
  --destroy \
  --to "$target" \
  | base64 -d > msg.boc
tonos-cli --url https://main.ton.dev sendfile msg.boc

# 2.1.3. All funds are collected to the target address. Contract is destroyed

# 2.2. More complex use case without contract destruction

# 2.2.1. Send TON to `$deposit_addr`

# 2.2.2. First transaction
collector msg "$secret"  \
  --init \
  --to "$target" \
  | base64 -d > msg.boc
tonos-cli --url https://main.ton.dev sendfile msg.boc

# 2.2.3. All funds are collected to the target address. 
# Contract is still in the network, but with zero balance. (seqno = 1) 

# 2.2.3. Send TON to `$deposit_addr`

# 2.2.4. Second transaction
collector msg "$secret"  \
  --seqno 1 \       # it is incremented after each successful transaction, 
  --to "$target" \  # so we should request it before each execution
  | base64 -d > msg.boc
tonos-cli --url https://main.ton.dev sendfile msg.boc

# 2.2.5. All funds are collected to the target address. 
# Contract is still in the network, but with zero balance. (seqno = 2) 

```
