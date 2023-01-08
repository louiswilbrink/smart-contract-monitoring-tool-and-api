# Contract Monitoring Tool & API

This service uses `ether-rs`'s RPC client to listen to transaction events on a specific contract address.

The service subscribes to transaction events and stores the following attributes in a Postgres database:

The service also exposes an API endpoint to query journaled transaction data, filtered on specific query parameters.

## Getting Started

Copy and rename the `.env-example` file, then edit with the appropriate values.

Make sure you have Postgres installed and running:

```
$ brew install postgresql
$ brew services start postgresql
```

The `transfers` table will automatically be created if it doesn't exist.

This repo can launch one of two different services:

```
// Defaults to running the contract monitoring service.
$ cargo run 
```

or

```
// Launches the API
$ cargo run -- --api
```

## API

### Quickstart

Here are a few requests to get warmed up (database will be seeded):

```
$ curl http://localhost:3000/transactions

// Response:
[
    {
        "tx_hash": "0x44f0d13225fbb7cee57ed0c45528d3959b5098fc",
        "sender": "0x33555f2008405660d04f128dc17e6ee01b77c4e7",
        "recipient": "0x33555f2008405660d04f128dc17e6ee01b77c4e7",
        "amount": 2.1214412019250876e20,
        "timestamp": 1673102315
    },
    {
        "tx_hash": "0x44f0d13225fbb7cee57ed0c45528d3959b5098fc",
        "sender": "0x33555f2008405660d04f128dc17e6ee01b77c4e7",
        "recipient": "0xaa1656b7d4629476fa4cf76ccfbc01a4653bac71",
        "amount": 4.0307382836576665e21,
        "timestamp": 1673102315
    },
    {
        "tx_hash": "0x74e74a0365bf33e94f65068a3cb86f7712ec59ad",
        "sender": "0x000000000035b5e5ad9019092c665357240f594e",
        "recipient": "0x00000000032962b51589768828ad878876299e14",
        "amount": 1.9014095536373356e21,
        "timestamp": 1673102375
    }
]
```

```
$ curl http://localhost:3000/transactions\?sender\=0x33555f2008405660d04f128dc17e6ee01b77c4e7\&recipient\=0xaa1656b7d4629476fa4cf76ccfbc01a4653bac71\&minAmount\=5\&maxAmount\=99999999999999999999999\&before\=1673102319\&order\=asc\&limit\=10\&offset\=0

// Response: 

[
    {
        "tx_hash": "0x44f0d13225fbb7cee57ed0c45528d3959b5098fc",
        "sender": "0x33555f2008405660d04f128dc17e6ee01b77c4e7",
        "recipient": "0xaa1656b7d4629476fa4cf76ccfbc01a4653bac71",
        "amount": 4.0307382836576665e21,
        "timestamp": 1673102315
    }
]
```

### `GET` /transactions

