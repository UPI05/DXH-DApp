DXH DApp
==================

## How to run:

First, deploy DXH_Token and save the contract ID. Then, you should change the TOKEN_SC_ADDR in `contract/src/lib.rs` and contractId in donate function of `frontend/near-interface.js`

```
yarn install
cd frontend && yarn install
cd ..
yarn deploy
yarn start
```
