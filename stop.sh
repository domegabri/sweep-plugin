#!/bin/bash

lightning-cli --network=regtest --lightning-dir=/tmp/l1 stop
lightning-cli --network=regtest --lightning-dir=/tmp/l2 stop
bitcoin-cli -regtest stop

#rm -rf /tmp/l1
#rm -rf /tmp/l2
