# Solana Lumberjack

A fully on-chain game where you chop wood. LFG 😤

## Development

**Start the local validator**

```sh
solana-test-validator
```

**Build the Solana program**

```sh
cd solana/lumberjack
cargo build-sbf
```

**Deploy Solana program**

```sh
cd solana/lumberjack
solana program deploy target/deploy/solana_lumberjack.so
```

**Run your game**

```sh
TURBO_SOL_SIGNER=<LOCAL_PRIVATE_KEY> turbo run -w .
```

