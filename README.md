# I am Human -- Proof of Humanity

Monorepository of contracts for the I am Human: proof of humanity protocol.

List of contracts:

- `sbt`: set of traits, events and common functions for [NEP-393](https://github.com/near/NEPs/pull/393/) SBT Standard.
- `registry`: implements the SBT Registry, documented in the [NEP-393](https://github.com/near/NEPs/pull/393/)
- `oracle`: SBT Issuer which relays on an off-chain authority signing claims for issuing SBTs.
- `demo-issuer`: basic SBT Issuer: contains a list of admins who are authorized to issue SBTs.

work in progress:

- `community-sbt`: Community Issuer of SBT tokens
- `soulbound-class`: An algebraic class of tokens to efficiently query if a user have required subset of tokens.
- `ubi`: demo use case implementing universal basic income.

## Deployed contracts

### Mainnet

- **SBT registry**: `registry.i-am-human.near`
- **Fractal**: `fractal.i-am-human.near`, verification pubkey base64: `"zqMwV9fTRoBOLXwt1mHxBAF3d0Rh9E9xwSAXR3/KL5E="`

Deprecated:

- **GoodDollar-SBT**: `gooddollar-v1.i-am-human.near`.
  verification pubkey base64: `"zqMwV9fTRoBOLXwt1mHxBAF3d0Rh9E9xwSAXR3/KL5E="`

### Testnet

- **SBT registry**: `registry-1.i-am-human.testnet`
- **Demo SBT Issuer**: `sbt1.i-am-human.testnet` (the `demo_issuer` contract)
- **Fractal Issuer**: `i-am-human-staging.testnet` (the `oracle` contract). Verification pubkey base64: `zqMwV9fTRoBOLXwt1mHxBAF3d0Rh9E9xwSAXR3/KL5E=`, `claim_ttl`: 3600ms
  - FV class: 1
- **Community-SBT**: `community-v1.i-am-human.testnet`
  - OG class: 1

Deprecated:

- `registry.i-am-human.testnet`
- GoodDollar SBT: `gooddollar-v1.i-am-human.testnet`
