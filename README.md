# Cw-20 Token Implementation

1. Configure `.env` and `package.json` to allow Node.js and NPM servers.
   Specifically, the env should include:
   * the encrypted mnemonic
   * the server's RPC and LCD URLs
   * the token's denom, the blockchain's ID (important, in our case we'll use a testnet)
   * the derivation path
   * and the gas price.
2. Deploy smart contract with `cwtools`. This has to do with the initial balance. The following command should work:
   ```bash
   cwtools wasm deploy eames-token --env .env --input '{"name": "Eames", "symbol": "EAM", "decimals": 10, "initial_balances": [{"amount: "20000", "address": "your_addr"}]}'
   ```
   * `eames-token` is just the name of the smart contract in `contract.rs`. It is a positional, so no need for tags.
   * `--env` option should precede the path to your environment file.
   * `--input` is the json schema for deploying the smart contract, or it can also be in base-16 format.
3. We can query the balance of our address. Otherwise, we will execute mint:
   ```bash
   cwtools wasm execute [your_addr] --env .env --input '{"mint":{"recipient": "your_addr", "amount": "1500"}}'
   ```