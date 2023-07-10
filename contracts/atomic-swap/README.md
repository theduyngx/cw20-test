# Atomic Swaps

**NOTE**: This is implemented by CosmWasm, source: https://github.com/CosmWasm/cw-tokens. Comments and most documents are,
however, added by the repository's author. That said, the codes will soon be modified to increase some flexibility and
adaptability.


----------------
## Mechanism:

Atomic swap is P2P, with a definitive sender and recipient. This implementation allows users to execute atomic swaps for
both **native** and **Cw20 tokens**. It is one-sided, but the other side can be realized by an equivalent (or identical) 
contract on any other blockchain (typically a different one).

Each side of an atomic swap has a sender, a recipient, a hash, and a timeout. It also has a unique id (for future calls to
reference it). The hash is a sha256-encoded 32-bytes long phrase. The timeout can be either time-based (seconds since
midnight, January 1, 1970), or block height based.

### Initiate & Lock:
* The initiator will initiate a swap by providing a hash of their secret preimage, and send some tokens (which will be locked
on the contract until any other end passes this hash in to release and confirm swap), and set an expiration.
* Before this timeout, the recipient can, likewise, simply copy the initiator's hash and similarly create a swap offer to the
initiator with the same hash.
* `Receive` in this implementation works identically to `Create`, though it is used specifically for Cw20 tokens, while
`Create` is used for simple, native tokens.

### Release:
* At which point where both sides have locked their tokens (`Create` swap offers), comes the `Release` phase. By pubicizing 
the preimage, the initiator has enabled both parties to finally be able to release each other's tokens with said preimage.
* We can think of the preimage as a password to unlock the frozen tokens. As such, the term 'release' refers to releasing the 
lock on smart contract for initiator's sent fund. This is the rationale behind the name Hash TimeOut Lock Contract (HTLC) for 
atomic swaps.

### Refund:
* As discussed, once the balance has been sent to the atomic swap contract, it is locked up there. This provides a safety
protocol of not allowing non-atomicity in the transaction, where the release phase is of initiator's full authorization.
* Because of this, `Refund` is simply not possible, as long as the swap has not expired. Meaning their tokens will be locked
unless released (to the other end's contract) or timed out.

See the [IOV atomic swap spec](https://github.com/iov-one/iov-core/blob/master/docs/atomic-swap-protocol-v1.md)
for details.


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

Or for a production-ready (optimized) build, run a build command in the
the repository root: https://github.com/CosmWasm/cw-plus#compiling.


----------------
## Detailed usage

  ```bash
  # create
  cwtools wasm execute [hashed_ref] --env .env --input '{
    "create": {
      "id": "some_id", 
      "hash": "4d9dbecbaaf42653d09a95c7e1986a047ce98afab5f9f8a4f98b20aa9913c984", 
      "recipient": "...", 
      "expires": {"at_height": 22222222222}
    }
  }' --amount "120023"

  # receive
  cwtools wasm execute [hashed_ref] --env .env --input '{
    "receive": {
      "sender": "...", 
      "amount": "120023", 
      "msg": "eyJjcmVhdGUiOnsiaWQiOiJzb21lX2lkIiwiaGFzaCI6IjRkOWRiZWNiYWFmNDI2NTNkMDlhOTVjN2UxOTg2YT
              A0N2NlOThhZmFiNWY5ZjhhNGY5OGIyMGFhOTkxM2M5ODQiLCJyZWNpcGllbnQiOiJvcmFpMXRjZW5xazRmMjZ2
              ZHo5N2V3ZGZjZWZyM2FrbnR6Z2h4ajdnY2F3IiwiZXhwaXJlcyI6eyJhdF9oZWlnaHQiOjIyMjIyMjIyfX19"
    }
  }'

  # release
  cwtools wasm execute [hashed_ref] --env .env --input '{
    "release": {
      "id": "some_id",
      "preimage": "this is a preimage of the first create message from the atomic swap"
    }
  }'
  ```

### Create:
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

### Receive:
  * This is identical to Create, although it is used for Cw20 tokens.
  * `sender` is the initiator. We can see that `amount` in this case is no longer an optional, but an argument in the JSON format
    of the `Cw20ReceiveMsg`.
  * `msg` is the create message in *binary*. That said, as we can see here, the message is in Base64 format. This is due to some
    confusion by the fact that JS actually converts this to binary. To get this `msg`, simply go to an online base64 encoder and
    encode the JSON format of the `--input` in `create`. Or use codes (see more in `crate::test::print_binary`).

### Release:
  * `id` should be the same swap id that the initiator sets it to. We are releasing this swap with the specified id, after all.
  * `preimage` is, as the name suggests, the preimage of the hash given to the swap.