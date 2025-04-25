# Sol Mtree

  This repository contains the Solana programs for the solana merkle tree implementation.

## Building and Testing

  ```sh
  cargo-build-sbf
  cargo test-sbf
  ```
  
## Deploy and rub

Start the solana localnet

```sh
solana-test-validator
```

Airdrop some sol to your localnet
```sh
solana airdrop 100 -u localhost
```

Build the program

```sh
cargo build-sbf
```

Deploy the program and copy the program id.

```sh
solana program deploy ./target/sbf-solana-solana/release/solana_program_mtree.so
```
  
Replace the program id with the program id you copied and run program.
```sh
cargo run --bin client -- insert-leaf  -p 8JakjgFi7PuFyMcHscqsx3o6rzsrHWUJ5ckTknuA1RW9 "hello world"
```