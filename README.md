# DeFi Hackathon 2019
## MultiDao
A Multi-Chain Assets Collateralized Stable Coin backed by IBC
## Structure
```
├── README.md
├── atom // An Cosmos-sdk chain with token transfer function
├── atom-plugin // TODO
├── dot // An Substrate chain with IBC token transfer function
├── dot-apps // Frontend of dot that can send token through IBC
├── dot-plugin // wasm plugin for other chains to verify the IBC package from dot chain
├── multidao  // MultiDao 
├── multidao-apps // Frontend of MultiDao
├── relayer // Relayer that listen to a chain's IBC packet and relay it to the target chain
├── substrate // Modified Substrate framework for proof generation and verification
└── substrate-subxt // Modified substrate rpc client
```

## Instructions
```
$ cd dot
$ cargo build --release
$ cd multidao
$ cargo build --release
$ cd relayer
$ cargo build --release
$ cd dot-plugin
$ cargo build --release
$ cd dot-apps
$ yarn
$ yarn start
$ cd multidao-apps
$ yarn
$ yarn start
$ cd dot
$ ./target/release/dot --base-path /tmp/chain-a --port 30333 --ws-port 9944 --dev
$ cd multidao
$ ./target/release/multidao --base-path /tmp/chain-b --port 30334 --ws-port 9945 --dev
// register dot-plugin 
$ cd relayer
$ ./target/release/relayer run --addr1 127.0.0.1:9944 --addr2 127.0.0.1:9945
// IBC transfer dot from dot frontend
// check console logs
// check multidao frontend
// exchange CDai
```
