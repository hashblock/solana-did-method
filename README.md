
# solana-did-method

**WORK IN PROGRESS --- BARELY FUNCTIONAL AND MISSING MUCH TO BE DONE**

# Building

## **1. Install rustc, cargo and rustfmt.**

```bash
$ curl https://sh.rustup.rs -sSf | sh
$ source $HOME/.cargo/env
$ rustup component add rustfmt
```

When building the master branch, please make sure you are using the latest stable rust version by running:

```bash
$ rustup update
```

Note that if this is not the latest rust version on your machine, cargo commands may require an [override](https://rust-lang.github.io/rustup/overrides.html) in order to use the correct version.

On Linux systems you may need to install libssl-dev, pkg-config, zlib1g-dev, etc.  On Ubuntu:

```bash
$ sudo apt-get update
$ sudo apt-get install libssl-dev libudev-dev pkg-config zlib1g-dev llvm clang make
```

Finally, install the latest [Solana CLI Suite](https://docs.solana.com/cli/install-solana-cli-tools)

## **2. Clone the solana-did-method repo.**

Using https
```bash
$ git clone https://github.com/hashblock/solana-did-method
```

Using ssh
```bash
$ git clone git@github.com:hashblock/solana-did-method.git
```

## **3. Build.**

First the Solana program
```bash
$ cd solana-did-method/program
$ cargo build-bpf
$ cd ..
```

Then the command line wallet
```bash
$ cargo build
```

## **4. Testing**

**Run the non-bpf test suite:**
This will start a local solana validator node (`solana-test-validator`)and run through tests in `src/main.rs`
```bash
$ cargo test -- --test-threads=1 --nocapture
```

## **5. Running**
WIP
