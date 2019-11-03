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
