# Integrations

## Freighter Deep-Link for `claim_bounty`

### Overview

[Freighter](https://www.freighter.app/) is the most widely used Stellar browser
wallet. It supports a transaction deep-link scheme that lets a web page open
Freighter with a pre-built transaction ready for the user to review and sign —
no manual transaction construction required.

Using this scheme, the MergeMint frontend can render a single "Claim" button
that, when clicked, opens Freighter with the correct `claim_bounty` invocation
pre-filled.

---

### Prerequisites

| Requirement | Notes |
|---|---|
| Freighter ≥ 5.5.0 | Earlier versions do not support the `stellar:` URI scheme |
| Stellar SDK (JS) | `@stellar/stellar-sdk` ≥ 12.x for XDR helpers |
| Network | Testnet (`Test SDF Network ; September 2015`) or Mainnet (`Public Global Stellar Network ; September 2015`) |

---

### The `stellar:` URI scheme

Freighter recognises URIs of the form:

```
stellar:<base64url-encoded XDR transaction envelope>
```

The XDR envelope encodes a `TransactionEnvelope` containing a single
`InvokeHostFunction` operation that calls `claim_bounty` on the MergeMint
contract.

---

### XDR transaction structure

The `InvokeHostFunction` operation must supply:

| Field | Value |
|---|---|
| `contract_id` | The deployed MergeMint contract address |
| `function` | `claim_bounty` |
| `args[0]` | `ScVal::Address` — the contributor's Stellar account address |
| `args[1]` | `ScVal::Bytes(32)` — the `BytesN<32>` bounty ID |

No footprint override is needed; Soroban simulation resolves the read/write
set automatically when the link is opened by Freighter.

---

### Generating the deep-link (JavaScript)

```js
import {
  Contract,
  Networks,
  TransactionBuilder,
  BASE_FEE,
  nativeToScVal,
  xdr,
} from "@stellar/stellar-sdk";
import { SorobanRpc } from "@stellar/stellar-sdk";

const RPC_URL = "https://soroban-testnet.stellar.org"; // or mainnet RPC
const CONTRACT_ID = "C..."; // deployed MergeMint contract address
const NETWORK_PASSPHRASE = Networks.TESTNET; // or Networks.PUBLIC

/**
 * Build and return a `stellar:` deep-link that opens Freighter with a
 * pre-filled claim_bounty transaction.
 *
 * @param {string} contributorAddress  - G... Stellar account of the contributor
 * @param {Uint8Array} bountyId        - 32-byte bounty ID
 * @returns {Promise<string>}          - the full `stellar:` URI
 */
export async function buildClaimBountyLink(contributorAddress, bountyId) {
  const server = new SorobanRpc.Server(RPC_URL);
  const account = await server.getAccount(contributorAddress);

  const contract = new Contract(CONTRACT_ID);

  const contributorScVal = nativeToScVal(contributorAddress, { type: "address" });
  const bountyIdScVal = xdr.ScVal.scvBytes(Buffer.from(bountyId));

  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(
      contract.call("claim_bounty", contributorScVal, bountyIdScVal)
    )
    .setTimeout(300) // 5-minute signing window
    .build();

  // Simulate to populate the transaction footprint (Soroban requirement)
  const simResult = await server.simulateTransaction(tx);
  if (SorobanRpc.Api.isSimulationError(simResult)) {
    throw new Error(`Simulation failed: ${simResult.error}`);
  }
  const preparedTx = SorobanRpc.assembleTransaction(tx, simResult).build();

  const xdrEnvelope = preparedTx.toEnvelope().toXDR("base64");
  // base64url-encode (replace + → -, / → _, strip =)
  const base64url = xdrEnvelope
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");

  return `stellar:${base64url}`;
}
```

---

### Example button (React)

```jsx
import { buildClaimBountyLink } from "./stellarUtils";

function ClaimButton({ bountyId, contributorAddress }) {
  const handleClaim = async () => {
    const link = await buildClaimBountyLink(contributorAddress, bountyId);
    window.location.href = link; // Freighter intercepts the stellar: scheme
  };

  return <button onClick={handleClaim}>Claim Bounty</button>;
}
```

---

### Manual verification on testnet

1. Deploy the contract to Testnet and note the contract address.
2. Call `create_bounty` to create a test bounty and capture the returned
   32-byte ID.
3. Run `buildClaimBountyLink` with a funded Testnet contributor address and the
   bounty ID.
4. Open the resulting `stellar:` URI in a browser with Freighter installed.
5. Confirm that Freighter opens the "Review transaction" screen showing a call
   to `claim_bounty` with the correct contract, contributor, and bounty ID.
6. Approve the transaction and verify on
   [Stellar Expert (Testnet)](https://stellar.expert/explorer/testnet) that the
   bounty status changed to `in_progress`.

---

### Limitations

- **Freighter version**: The `stellar:` URI scheme requires Freighter ≥ 5.5.0.
  Users on older versions will see a blank page or no response.
- **Network selection**: Freighter determines the network from the transaction's
  `networkPassphrase`. If the user's Freighter is set to a different network the
  transaction will be rejected at signing time.
- **Account must exist on-chain**: `TransactionBuilder` requires a sequence
  number, so the contributor account must be funded before the link is generated.
  For Testnet, use the
  [Friendbot](https://friendbot.stellar.org/?addr=<contributor_address>) to fund
  new accounts.
- **Deep-link size**: Very large XDR envelopes (rare for single-operation
  invocations) may exceed browser URL length limits. The assembled Soroban
  transaction for `claim_bounty` is well within typical limits (~2–4 KB).
