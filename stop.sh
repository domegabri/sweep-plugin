#!/bin/bash

if [ -z "$NETWORK" ]; then
	export NETWORK=regtest
fi

lightning-cli --network=$NETWORK --lightning-dir=/tmp/l1 stop
lightning-cli --network=$NETWORK --lightning-dir=/tmp/l2 stop
bitcoin-cli -$NETWORK stop

#rm -rf /tmp/l1
#rm -rf /tmp/l2
