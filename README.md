# Cw-20 Smart Contract Implementation

`cwtools` is an alternative to Rust-optimizer. It is developed by Oraichain, source code here:
https://github.com/oraichain/cosmwasm-tools (I did NOT develop this tool, the source code belongs to Oraichain - though in this
version, there are minor changes within the code). This file only serves as a reminder of the steps and brief functionalities
to 'deploy' a Cosmos smart contract.

----------------
## Configuration

Configure `.env` and `package.json` to allow Node.js and NPM servers (samples of these files are in this respository) to manage
your crate. Specifically, the env should include:
  * the encrypted mnemonic
  * the server's RPC and LCD URLs
  * the token's denom, the blockchain's ID (important, in our case we'll use a testnet)
  * the derivation path
  * and the gas price.

----------------
## Instantiate
Normally, you'd deploy your smart contract, which you can do with `cwtools`. That said, deployment is expensive, so if your
contract is reusing a code base from some other contract already existed on the network, it is better to instantiate it without
uploading (the latter can therefore be optional). This is where CosmWasm differs from EVM - you don't actually have to deploy a
source code for a smart contract with existing logic!<br>

  ```bash
  # Instantiate
  cwtools wasm instantiate --env .env --input '{
    "name": "Eames", "symbol": "EAM", "decimals": 10, 
    "initial_balances": [{
      "amount: "20000", "address": "your_addr"
    }], 
    "mint": {"minter": "your_addr"}
  }' --code-id 6082 --label "basic" --admin "your_addr"
  ```
  * This process is so that you instantiate a copied wasm file from another existing contract with its specified code id.
  * `--env` option should precede the path to your environment file.
  * `--input` is the json schema for deploying the smart contract, or it can also be in base-16 format. Note that if this were
    for atomic swap instantiation, then the input would be an empty `{}` (see `instantiate` function in `contract.rs`).
  * `--code-id` is a source code's ID (true ID is a complicated hashed value, so they simplify this process by adding another
    mapping to an ever-incrementing integer identifier).
    * In simplier terms, though, it is the simplified id of the smart contract that you referred to, where said contract shares
      the same Cw20 base as yours.
    * Sharing the same underlying standard allows this instantiation without having to upload your source code to the network.
  * `--label` is required, though arbitrary.
  * `--admin` is used specifically for operations that require some form of authorization. It could be an existing contract's,
    or a wallet's address. In this case, we've used `"your_addr"`, which is referring to the wallet's address.

----------------
## Upload / Deploy
  ```bash
  # Compile
  cwtools wasm build contracts/atomic-swap -o path/to/compiled
  ```
  * The first step to upload or deploy is to compile your contract (at top level of smart contract crate) into a compressed
    `.wasm` file. This is the file that can be deployed / uploaded onto the blockchain network. (In this example, we use
    atomic-swap contract.)
  * `cwtools` will automatically optimize the file size for you with its `build` wasm operation. Full-size file is often large
    and unsuitable for the blockchain.

  ```bash
  # Upload
  cwtools wasm upload path/to/compiled/atomic-swap.wasm --env .env --input '{
    "name": "Eames", "symbol": "EAM", "decimals": 10, 
    "initial_balances": [{
      "amount: "20000", "address": "your_addr"
    }]
  }'

  # Deploy
  cwtools wasm deploy path/to/compiled/atomic-swap.wasm --env .env --input '{
    "name": "Eames", "symbol": "EAM", "decimals": 10, 
    "initial_balances": [{
      "amount: "20000", "address": "your_addr"
    }]
  }'
  ```
  * The upload process is to upload your smart contract and obtain the `.wasm` file to run on the network. As disucssed, you
    won't be required to upload the source code unless you make internal logic changes to the Cw20 base.
  * After uploading, a new smart contract ID (the simplified `code-id` discussed above) will be returned. You will use this id
    to **instantiate** your smart contract on the chain. This means that instead of borrowing some other contract's id, you are
    instead using your own true, new provided one from the uploaded source code.
  * Deploy is simply upload and instantiate in one. Just as the process above, it will first upload the `.wasm` file, use the
    returned `code-id` to pass it to instantiate the contract.

----------------
## Execute & Query
We can query, for example, the balance of our address. Otherwise, we can also execute. The following is a mint execution 
for basic smart contract example:
  ```bash
  cwtools wasm execute hashed_ref --env .env --input '{
    "mint": {"recipient": "your_addr", "amount": "1500"}
  }'
  ```
  * `hashed_ref` is the hashed reference to the instantiation smart contract you had borrowed.
  * Specifically, in instantitation, you called a `--code-id`, and that id is the smart contract whose underlying standard you
    borrowed so that you don't have to upload your source code to the network. When instantiation of your new smart contract
    succeeds, you will receive a successful `MsgInstantiation` on the blockchain, where it will specify a new hashed reference.
    Use that value here.
  * In our instantiate operation above, we specified a minter, which is our address. This is why this mint operation will work.

For details of atomic swap, read README of atomic-swap directory in contracts.