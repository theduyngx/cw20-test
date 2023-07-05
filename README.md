# Cw-20 Token Implementation

1. **Configuration**: configure `.env` and `package.json` to allow Node.js and NPM servers.
   Specifically, the env should include:
   * the encrypted mnemonic
   * the server's RPC and LCD URLs
   * the token's denom, the blockchain's ID (important, in our case we'll use a testnet)
   * the derivation path
   * and the gas price.
   <br><br>

2. **Upload**: normally, you'd deploy your smart contract, which you can do with `cwtools`. That said, deployment is expensive, so it is better to instantiate and then upload it (the latter is often optional and only required once when initiated). This is where CosmWasm differs from Ethereum - you don't actually have to deploy your newly designed smart contract!

   a. <u>Instantiate</u>
   ```bash
   cwtools wasm instantiate --env .env --input '{"name": "Eames", "symbol": "EAM", "decimals": 10, "initial_balances": [{"amount: "20000", "address": "your_addr"}]}' --code-id 6082
   ```
   * This process is so that you use your smart contract to verify a source code (sort of) and effectively getting a new smart contract state on the network.
   * `--env` option should precede the path to your environment file.
   * `--input` is the json schema for deploying the smart contract, or it can also be in base-16 format.
   * `--code-id` is a source code's ID (true ID is a complicated hashed value, so sometimes they can simplify this process by adding another mapping to an ever-incrementing integer identifier).<br><br>

   b. <u>Upload</u>
   ```bash
   cwtools wasm upload ~/Desktop/Oraichain/cw20-test --env .env --input '{"name": "Eames", "symbol": "EAM", "decimals": 10, "initial_balances": [{"amount: "20000", "address": "your_addr"}]} '
   ```
      * This process is so that you upload your smart contract and obtain the `.wasm` file to run on the network.
      <br><br>
   
   c. <u>Deploy</u>
   ```bash
   cwtools wasm deploy eames-token --env .env --input '{"name": "Eames", "symbol": "EAM", "decimals": 10, "initial_balances": [{"amount: "20000", "address": "your_addr"}]}'
   ```
   * `eames-token` is just the name of the smart contract in `contract.rs`. It is a positional, so no need for tags.
   <br><br>

3. **Execute & Query**: We can query the balance of our address. Otherwise, we will execute mint:
   ```bash
   cwtools wasm execute [your_addr] --env .env --input '{"mint":{"recipient": "your_addr", "amount": "1500"}}'
   ```