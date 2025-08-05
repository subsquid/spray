# spray

Proxy service for [Yellowstone Geyser gRPC plugin](https://github.com/rpcpool/yellowstone-grpc),
that exposes transactions and blocks via JSON-RPC Websocket subscription 
with advanced filtering capabilities.

## Features

* Subsquid portal-like query format for data filtering
* Subsquid portal-like data format for data messages
* Multiple geyser endpoints can be configured for reliable and fast data delivery

## API

Only a single subscription kind is available.

* `spraySubscribe` - subscription method, accepts [data filter](#data-filter) as a single parameter
* `sprayNotification` - [data message](#data-message)
* `sprayUnsubscribe` - subscription cancellation

### Data filter

Data filter follows [Subsquid portal Solana data query format](https://docs.sqd.ai/solana-indexing/network-api/solana-api/#data-requests) 
except that block range selection parameters (`fromBlock`, `toBlock`, `parentBlockHash`) 
are not available and forbidden.

### Data message

There are two kinds of data messages - block notification and transaction notification.

```ts
interface TransactionNotification {
    type: 'transaction'
    slot: number
    transactionIndex: number
    // Data items matched by the data filter
    transaction?: SubquidPortalSolanaTransaction
    instructions?: SubquidPortalSolanaInstruction[]
    balances?: SubquidPortalSolanaBalance[]
    tokenBalances?: SubquidPortalSolanaTokenBalance[]
}

interface BlockNotification {
    type: 'block'
    slot: number
    // Solana block info defined by `query.fields.block`
    header?: SolanaPortalBlockHeader
}
```

Block notifications are pushed to the client in the following circumstances

* Transaction notification belonging to the given block was pushed before
* Every fifth slot since the last pushed block or subscription start
* Every block if `query.includeAllBlocks` is `true`.

## Setup

```
Usage: sqd-spray <config>

Arguments:
  <config>  Config file
```

Where config file has the following format (yaml)

```yaml
port: 3000 # port to listen on (optional, default is 3000)
# data sources
sources:
  getblock: # data source name
    url: https://go.getblock.io/
    x_access_token: xxx # add `X-Access-Token` header to gRPC request

  shyft:
    url: https://xxx
    x_token: xxx  # add `X-Token` header to every gRPC request
```