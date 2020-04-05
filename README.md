This plugin adds the sweep rpc method to sweep paper wallets.

Currently working in testnet (uses blockstream.info API).

To build:

```cargo build```

Usage:

```lightning-cli sweep <private-key> <destination-address>```

Returns signed transaction sending coins to the destination address