# Contract Monitoring Tool & API



This service uses `ether-rs`'s RPC client to listen to transaction events on a specific contract address.



The service subscribes to transaction events and stores the following attributes in a Postgres database:



The service also exposes an API endpoint to query journaled transaction data, filtered on specific query parameters.



## Getting Started



Copy and rename the `.env-example` file, then edit with the appropriate values.



Make sure you have Postgres installed and running:



```bash
$ brew install postgresql
$ brew services start postgresql
```



The `transfers` table will automatically be created (and seeded) if it doesn't exist.



This repo can launch one of two different services:



```bash
# Defaults to running the contract monitoring service.
$ cargo run

# Launches API
$ cargo run -- --api
```



## API



### Quickstart



Here are a few requests to get warmed up (database will be seeded):



```bash
# Get all transactions
$ curl http://localhost:3000/transactions

[
    {
        "tx_hash": "0x87327201eac4cf8df4ec4831e434ffd872b7b4eb",
        "sender": "0x00000000a991c429ee2ec6df19d40fe0c80088b8",
        "recipient": "0xc5be99a02c6857f9eac67bbce58df5572498f40c",
        "amount": 34005500222.0,
        "timestamp": 1673102315
    },
    {
        "tx_hash": "0xb653d23e95a02cb6de0bef363406ee617a545cd4",
        "sender": "0x62716cd006b3c64ca1ef5dc439b56069e16cbe9c",
        "recipient": "0x5c6eff81d09cb1ebe7ce87f4f2df638f83b05b6c",
        "amount": 464332344332.0,
        "timestamp": 1673101315
    },
    {
        "tx_hash": "0x1f7259c7940891146dd6888d7f707b4a889844d7",
        "sender": "0x8c0f2dbabe1fe6d79bbc90930de313da9c81c8bd",
        "recipient": "0xf267d62188820f1ce7aebf01bfdefc5d4c45d3cf",
        "amount": 77238333.0,
        "timestamp": 1673101200
    }
]
```



```bash
# Filter transactions
$ curl http://localhost:3000/transactions?sender=0x00000000a991c429ee2ec6df19d40fe0c80088b8&recipient=0xc5be99a02c6857f9eac67bbce58df5572498f40c&minAmount=5&maxAmount=99999999999999999999999&before=1673102319&order=asc&limit=10&offset=0

[
    {
        "tx_hash": "0x87327201eac4cf8df4ec4831e434ffd872b7b4eb",
        "sender": "0x00000000a991c429ee2ec6df19d40fe0c80088b8",
        "recipient": "0xc5be99a02c6857f9eac67bbce58df5572498f40c",
        "amount": 34005500222.0,
        "timestamp": 1673102315
    }
]
```



```bash
# Find transaction by hash
$ curl http://localhost:3000/transactions/0x87327201eac4cf8df4ec4831e434ffd872b7b4eb

{
    "tx_hash": "0x87327201eac4cf8df4ec4831e434ffd872b7b4eb",
    "sender": "0x00000000a991c429ee2ec6df19d40fe0c80088b8",
    "recipient": "0xc5be99a02c6857f9eac67bbce58df5572498f40c",
    "amount": 34005500222.0,
    "timestamp": 1673102315
}
```



## List Transactions

**Request URI**

**GET** `http://localhost:3000/transactions`

##### Query parameters

| Name        | Type                                                         |
| ----------- | ------------------------------------------------------------ |
| `sender`    | Ethereum address, i.e. `"0x00000000a991c429ee2ec6df19d40fe0c80088b8"` |
| `recipient` | Ethereum address, i.e. `"0xc5be99a02c6857f9eac67bbce58df5572498f40c"` |
| `minAmount` | Float or Integer                                             |
| `maxAmount` | Float or Integer                                             |
| `before`    | Timestamp (in seconds)                                       |
| `after`     | Timestamp (in seconds)                                       |
| `order`     | Enum: `asc` or `desc`                                        |
| `limit`     | Integer                                                      |
| `offset`    | Pagination (Integer)                                         |

**Request Headers**

No request headers necessary.

**Request Body**

No request body.

**Request Notes**

This request is idempotent and automatic retries initiated by the client implementation is allowed.

**Response**

Upon a successful request, returns a list of `transactions` from the database.  If no transactions have been created yet, returns an empty list.  Here are the keys and descriptions for the `transaction` entity object:

| Key         | Description                                                  |
| ----------- | ------------------------------------------------------------ |
| `tx_hash`   | `string`; A unique, durable identifier for transaction.      |
| `sender`    | `string`; Sender's public address                            |
| `recipient` | `string`; Recipient's public address                         |
| `amount`    | `float`; Represents the amount of wei sent in the transaction. |
| `timestamp` | `timestamp`; Represented in seconds since epoch.  Notes the block time that the transaction was included on |

Example Response `body`:

```
[
    {
        "tx_hash": "0x87327201eac4cf8df4ec4831e434ffd872b7b4eb",
        "sender": "0x00000000a991c429ee2ec6df19d40fe0c80088b8",
        "recipient": "0xc5be99a02c6857f9eac67bbce58df5572498f40c",
        "amount": 34005500222.0,
        "timestamp": 1673102315
    },
    {
        "tx_hash": "0xb653d23e95a02cb6de0bef363406ee617a545cd4",
        "sender": "0x62716cd006b3c64ca1ef5dc439b56069e16cbe9c",
        "recipient": "0x5c6eff81d09cb1ebe7ce87f4f2df638f83b05b6c",
        "amount": 464332344332.0,
        "timestamp": 1673101315
    },
    {
        "tx_hash": "0x1f7259c7940891146dd6888d7f707b4a889844d7",
        "sender": "0x8c0f2dbabe1fe6d79bbc90930de313da9c81c8bd",
        "recipient": "0xf267d62188820f1ce7aebf01bfdefc5d4c45d3cf",
        "amount": 77238333.0,
        "timestamp": 1673101200
    }
]
```

**Server Response Codes:**

| Code  | Body                      | Meaning                                                      |
| ----- | ------------------------- | ------------------------------------------------------------ |
| `200` | `Array` of `transactions` | The request was successful and a list of  `transaction` entities were return |
| `500` | None                      | `Internal Server Error`; The server encountered an error that terminated execution before a proper response was determined. |



## Find Transaction

**Request URI**

**GET** `http://localhost:3000/transactions/:txhash`

##### Query parameters

No query parameters.

**Request Headers**

No request headers necessary.

**Request Body**

No request body.

**Request Notes**

This request is idempotent and automatic retries initiated by the client implementation is allowed.

**Response**

Upon a successful request, returns a single `transaction`.  Here are the keys and descriptions for the `transaction` entity object:

| Key         | Description                                                  |
| ----------- | ------------------------------------------------------------ |
| `tx_hash`   | `string`; A unique, durable identifier for transaction.      |
| `sender`    | `string`; Sender's public address                            |
| `recipient` | `string`; Recipient's public address                         |
| `amount`    | `float`; Represents the amount of wei sent in the transaction. |
| `timestamp` | `timestamp`; Represented in seconds since epoch.  Notes the block time that the transaction was included on |

Example Response `body`:

```
{
    "tx_hash": "0x87327201eac4cf8df4ec4831e434ffd872b7b4eb",
    "sender": "0x00000000a991c429ee2ec6df19d40fe0c80088b8",
    "recipient": "0xc5be99a02c6857f9eac67bbce58df5572498f40c",
    "amount": 34005500222.0,
    "timestamp": 1673102315
}
```

**Server Response Codes:**

| Code  | Body          | Meaning                                                      |
| ----- | ------------- | ------------------------------------------------------------ |
| `200` | `transaction` | The request was successful a single  `transaction` is returned. |
| `500` | None          | `Internal Server Error`; The server encountered an error that terminated execution before a proper response was determined. |

