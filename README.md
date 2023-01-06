# spearbit-take-home

This service uses `ether-rs`'s RPC client to listen to transaction events on a contract address.

Make sure to copy and rename the `.env-example` file.

The service subscribes to transaction events and stores the following attributes in a Postgres database:

* `txhash`
* `sender`
* `recipient`
* `amount`
* `timestamp`

The service also exposes an API endpoint to query journaled transaction data, filtered on specific query parameters.

### `GET` /transactions

