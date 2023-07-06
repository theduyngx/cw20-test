# Cw-20 Token Implementation

`cwtools` is an alternative to Rust-optimizer. It is developed by Oraichain, source code here: https://github.com/oraichain/cosmwasm-tools (I did NOT develop this tool, the source code belongs to Oraichain - though in this version, there are minor changes within the code). This file only serves as a reminder of the steps and brief functionalities to 'deploy' a Cosmos smart contract.

----------------
## Configuration

Configure `.env` and `package.json` to allow Node.js and NPM servers (samples of these files are in this respository). Specifically, the env should include:
   * the encrypted mnemonic
   * the server's RPC and LCD URLs
   * the token's denom, the blockchain's ID (important, in our case we'll use a testnet)
   * the derivation path
   * and the gas price.

----------------
## Instantiate
Normally, you'd deploy your smart contract, which you can do with `cwtools`. That said, deployment is expensive, so it is better to instantiate it without uploading (the latter is often optional). This is where CosmWasm differs from EVM - you don't actually have to deploy a source code for a new smart contract!<br>

What this implies is you won't even need a Rust Cw20 source code to begin with! You're simply borrowing the implementation of an already existed contract on the network without having to upload your code, effectively saving space and gas.

   ```bash
   cwtools wasm instantiate --env .env --input '{"name": "Eames", "symbol": "EAM", "decimals": 10, "initial_balances": [{"amount: "20000", "address": "your_addr"}], "mint": {"minter": "your_addr"}}' --code-id 6082  --label "eam"
   ```
   * This process is so that you use your smart contract to verify a source code (sort of) and effectively getting a new smart contract state on the network.
   * `--env` option should precede the path to your environment file.
   * `--input` is the json schema for deploying the smart contract, or it can also be in base-16 format.
   * `--code-id` is a source code's ID (true ID is a complicated hashed value, so they simplify this process by adding another mapping to an ever-incrementing integer identifier).
      * In simplier terms, though, it is the simplified id of the smart contract that you referred to, where said contract shares the same Cw20 base as yours.
      * Sharing the same underlying standard allows this instantiation without having to upload your source code to the network.
   * `--label` is required, though arbitrary.

----------------
## Upload / Deploy (optional)
   ```bash
   # Compile
   cwtools wasm build contracts/atomic-swap -o path/to/compiled
   ```
   * The first step to upload or deploy is to compile your contract (at top level of crate) into a compressed `.wasm` file. This is the file that can be deployed / uploaded onto the blockchain network. (In this example, we use atomic-swap contract.)
   * `cwtools` will automatically optimize the file size for you with its `build` wasm operation. Full-size file is often large and unsuitable for the blockchain.

   ```bash
   # Upload
   cwtools wasm upload path/to/compiled/atomic-swap.wasm --env .env --input '{"name": "Eames", "symbol": "EAM", "decimals": 10, "initial_balances": [{"amount: "20000", "address": "your_addr"}]} '

   # Deploy
   cwtools wasm deploy path/to/compiled/atomic-swap.wasm --env .env --input '{"name": "Eames", "symbol": "EAM", "decimals": 10, "initial_balances": [{"amount: "20000", "address": "your_addr"}]}'
   ```

   * The upload process is to upload your smart contract and obtain the `.wasm` file to run on the network. That said, you won't be required to upload this unless you make internal logic changes to the Cw20 base.
   * Deploy is simply upload and instantiation in one.
   * With this process, a brand new smart contract source code has been pushed to the network and ready to be used.

----------------
## Execute & Query
We can query the balance of our address. Otherwise, we can also execute. The following is a mint execution example:
   ```bash
   cwtools wasm execute hashed_ref --env .env --input '{"mint":{"recipient": "your_addr", "amount": "1500"}}'
   ```
   * `hashed_ref` is the hashed reference to the instantiation smart contract you had borrowed.
   * Specifically, in instantitation, you called a `--code-id`, and that id is the smart contract whose underlying standard you borrowed so that you don't have to upload your source code to the network. When instantiation of your new smart contract succeeds, you will receive a successful `MsgInstantiation` on the blockchain, where it will specify a new hashed reference. Use that value here.