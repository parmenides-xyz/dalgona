# dalgona

`dalgona` is a Commonware-native Espresso data availability library, following
the same workspace shape as `coro`.

It is intended for applications that want to:

- submit opaque batch bytes to Espresso as namespaced transactions
- read them back by exact transaction reference
- verify returned bytes against Espresso transaction commitments
- build a single-sequencer layer above the DA client

It is not:

- Espresso-as-consensus
- a namespace-scanning indexer
- a host-chain state machine or account/signature framework

## Workspace Layout

- `dalgona/`: core library crate

## Core Layers

The crate is structured around the same plug points as `coro`:

- `backend`
- `types`
- `single_sequencer`

The current implementation starts with the Espresso-facing type surface:

- `SubmissionId`
- `TransactionRef`
- `TransactionKey`
- `PublishRequest`
- `PublishReceipt`
- `VerifiedTransaction`

`backend` and `single_sequencer` are present as module boundaries for the next
implementation pass.
