# Contract Monitoring Tool

This service uses `ether-rs`'s RPC client to listen to transaction events on a specified contract address.

The service subscribes to transaction events and stores the following attributes in a Postgres database:

* `txhash`
* `sender`
* `recipient`
* `amount`
* `timestamp`

The service also exposes an API endpoint to query journaled transaction data, filtered on specific query parameters.

### `GET` /transactions




## Getting Started

Copy and rename the `.env-example` file, then add appropriate values.

Make sure you have Postgres installed and running:

```
$ brew install postgresql
$ brew services start postgresql
```

Log into Postgres with the username/password you set in your `.env` file.  Create the `transfers` table:

```
# CREATE TABLE transfers (
    id          INTEGER NOT NULL UNIQUE PRIMARY KEY,
    tx_hash     VARCHAR(66) NOT NULL,
    sender      VARCHAR(44) NOT NULL,
    recipient   VARCHAR(44) NOT NULL,
    amount      FLOAT,
    timestamp   TIMESTAMP
);
```

Run!

```
$ cargo run
```
