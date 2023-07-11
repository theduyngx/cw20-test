# Atomic Swaps

**NOTE**: This is [implemented by CosmWasm](https://github.com/CosmWasm/cw-tokens). Comments and most documents are, however,
added by the repository's author. That said, the codes have been slightly and will continue to be modified to increase some
flexibility and adaptability.


----------------
## Mechanism

Atomic swap is P2P, with a definitive sender and recipient. This implementation allows users to execute atomic swaps for both
**native** and **Cw20 tokens**. It is one-sided, but the other side can be realized by an equivalent (or identical) contract
on any other blockchain (typically a different one). The implementation also does not allow same sender and recipient.
Migration is now in development.

Each side of an atomic swap has a sender, a recipient, a hash, and a timeout. It also has a unique id (for future calls to
reference it). The hash is a sha256-encoded 32-bytes long phrase. The timeout can be either time-based (seconds since
midnight, January 1, 1970), or block height based. The hash encodes a preimage, which is a UTF-8 string.

Note that it is not the smart contract's responsibility to ensure that the specific tokens in a swap agreement (BTC, ETH, etc.)
are of their correct type, or that the amount offered does not equal to the amount agreed upon by either end. It is also not
the contract's responsibility to release the tokens to the recipient, even if said recipient *forgets* passed due date (meaning
no automatic protocol to release the tokens).

### Initiate & Lock
* The initiator will initiate a swap by providing a hash of their secret preimage, and send some tokens (which will be locked
on the contract until the recipient passes this hash in to confirm swap and release), and set an expiration.
* Before this timeout, the recipient can, likewise, simply copy the initiator's hash and similarly create a swap offer to the
initiator with the same hash. This will lock the recipient's tokens as well, where the only action left available is to either
wait till expiration for a refund (see more in **Refund** segment), or release their tokens to the other end.
* `Receive` in this implementation works identically to `Create`, though it is used specifically for Cw20 tokens, while
`Create` is used for simple, native tokens.

### Release
* At which point where both sides have locked their tokens (`Create` swap offers), comes the `Release` phase. By publicizing 
the preimage, the initiator has enabled both parties to finally be able to release each other's tokens with said preimage.
* We can think of the preimage as a password to unlock the frozen tokens. As such, the term 'release' refers to releasing the 
lock on smart contract for initiator's sent fund. This is the rationale behind the name Hash TimeOut Lock Contract (HTLC) for 
atomic swaps.
* To reiterate, the only way to release the locked tokens is to either wait till expiration, or that the initiator agrees to
publicize their preimage, which would only occur when the initiator agrees that the recipient's swap offer is valid. By
doing so, `Release` may be executed, and the locked tokens will thus be transferred to their respective recipient.

### Refund
* As discussed, once the balance has been sent to the atomic swap contract, it is locked up there. This provides a safety
protocol of not allowing non-atomicity in the transaction, where the release phase is of initiator's full authorization.
* Because of this, `Refund` is simply not possible, as long as the swap has not expired. Meaning their tokens will be locked
unless released (to the other end's contract) or timed out.

See the [IOV atomic swap spec](https://github.com/iov-one/iov-core/blob/master/docs/atomic-swap-protocol-v1.md) for details.


----------------
## Running this contract

You will need Rust 1.44.1+ with `wasm32-unknown-unknown` target installed.

You can run unit tests on this via: 
```bash
cargo test
```

Once you are happy with the content, you can compile it to wasm via:
```bash
RUSTFLAGS='-C link-arg=-s' cargo wasm
cp ../../target/wasm32-unknown-unknown/release/cw20_atomic_swap.wasm .
ls -l cw20_atomic_swap.wasm
sha256sum cw20_atomic_swap.wasm
```

Or for a production-ready (optimized) build, run a build command in the the repository root: 
https://github.com/CosmWasm/cw-plus#compiling.


----------------
## Detailed usage

We will clarify some cwtools commands to use the services provided by the smart contract.

### Create
  ```bash
  cwtools wasm execute [hashed_ref] --env .env --input '{
    "create": {
      "id": "some_id", 
      "hash": "4d9dbecbaaf42653d09a95c7e1986a047ce98afab5f9f8a4f98b20aa9913c984", 
      "recipient": "...", 
      "expires": {"at_height": 22222222222}
    }
  }' --amount "120023"
  ```
  * `hashed_ref` is the hashed reference to the instantiation smart contract you had uploaded. Unlike the basic implementation,
    we had to upload this smart contract to the network, and in turn, we have created a brand new smart contract with, likewise,
    a hashed value referencing the newly created contract.
  * `id` is the swap ID which can really be anything named by the initiator. The swap storage on the contract is indexed by these
    simple, human-readable identifiers. Even if the id had already existed, we would just get an error.
  * `hash` is the hashed value of the preimage created by the initiator. The initiator, before creating a swap offer, will store
    their own secret private key, which will be hashed and passed here.
  * `recipient` is the recipient's wallet address. If the offer is approved (the preimage became known and swap can be released),
    the recipient receives the token. Remember that atomic swap is P2P.
  * `expires` is the expiration. In this example, we use `at_height`, which represents block height expiration.
  * `--amount` takes in an integer string. It is the amount that this initiator is asking to atomically swap
    for with the specified `recipient` above.

### Receive
  ```bash
  cwtools wasm execute [hashed_ref] --env .env --input '{
    "receive": {
      "sender": "...", 
      "amount": "120023", 
      "msg": "eyJjcmVhdGUiOnsiaWQiOiJzb21lX2lkIiwiaGFzaCI6IjRkOWRiZWNiYWFmNDI2NTNkMDlhOTVjN2UxOTg2Y
             TA0N2NlOThhZmFiNWY5ZjhhNGY5OGIyMGFhOTkxM2M5ODQiLCJyZWNpcGllbnQiOiJvcmFpMXRjZW5xazRmMjZ
             2ZHo5N2V3ZGZjZWZyM2FrbnR6Z2h4ajdnY2F3IiwiZXhwaXJlcyI6eyJhdF9oZWlnaHQiOjIyMjIyMjIyfX19"
    }
  }'
  ```
  * This is identical to Create, although it is used for Cw20 tokens.
  * `sender` is the initiator. We can see that `amount` in this case is no longer an optional, but an argument in the JSON format
    of the `Cw20ReceiveMsg`.
  * `msg` is the create message in *binary*. That said, we can see that the message is in Base64 format. This is due to the fact
    that JS actually, eventually does convert this to binary. To get this `msg`, simply go to an online base64 encoder and encode
    the JSON format of the `--input` in `create`. Or use codes (see more in `crate::test::print_binary`).

### Release
  ```bash
  cwtools wasm execute [hashed_ref] --env .env --input '{
    "release": {
      "id": "some_id",
      "preimage": "this is a preimage of the first create message from the atomic swap"
    }
  }'
  ```
  * `id` should be the same swap id that the initiator sets it to. We are releasing this swap with the specified id, after all.
  * `preimage` is, as the name suggests, the preimage of the hash given to the swap. In this example, the given preimage is, in
    fact, the preimage of the hash above.

### Refund
  ```bash
  cwtools wasm execute [hashed_ref] --env .env --input '{
    "refund": {
      "id": "some_id"
    }
  }'
  ```
  * `id` is the only requirement for refunding. Meaning refund is local to the smart contract itself.
  * Specifically, refund will delete the swap offer on the smart contract's storage through accessing the key `id` to delete the
    entry. Refunding when swap has not expired will return an error.

### Migrate
  ```bash
  cwtools wasm migrate [hashed_ref] --env ../.env --input '{}' --code-id 6012
  ```
  * Important note: migration requires authorization, meaning the field `--admin` is required when instantiating this contract.
  * In this migration command, we can see that we're referring to the `[hashed_ref]` of the smart contract (the contract's 
    address), and its code-id.
  * For now, an input is not required for migration (as with most Cw20 standards).
<br><br>

**NOTE:** Due to some environment incompatibilites in the configuration of cwtools, the `cwtools` directory in this repository
has made some changes. If your cwtools cannot run, use this repository's cwtools instead (which in turn will not be global). To
do so, `cd` to `cwtools` directory within this repository, and instead of running the command `cwtools`, run `yarn start:dev`
instead. Of course, within this directory, your `.env` file will be in the parent's. Here's an example:
```bash
yarn start:dev wasm execute [hashed_ref] --env ../.env --input '{ ... }'
```