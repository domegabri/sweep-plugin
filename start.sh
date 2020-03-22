#!/bin/bash

export RUST_BACKTRACE=full

#create new plugin log file
rm /tmp/pluginlog
touch /tmp/pluginlog
export PLUGIN=$PWD/target/debug/simpleplugin

# start bitcoind 
bitcoind -regtest -txindex --daemon
# node 1: datadir and config
mkdir /tmp/l1
echo log-level=debug >> /tmp/l1/config
echo log-file=/tmp/l1/log >> /tmp/l1/config
echo allow-deprecated-apis=false >> /tmp/l1/config

# node 2: datadir and config
mkdir /tmp/l2
echo log-level=debug >> /tmp/l2/config
echo log-file=/tmp/l2/log >> /tmp/l2/config
echo allow-deprecated-apis=false >> /tmp/l2/config
# start lightning daemons
lightningd --daemon --network=regtest --lightning-dir=/tmp/l1
lightningd --daemon --network=regtest --lightning-dir=/tmp/l2 --bind-addr=/tmp/l2/peer --plugin=$PLUGIN

# set up aliases
alias l1="lightning-cli --network=regtest --lightning-dir=/tmp/l1"
alias l2="lightning-cli --network=regtest --lightning-dir=/tmp/l2"
alias bcli="bitcoin-cli -regtest"


# to debug plugin:
# $ mkfifo test
# $ RUST_BACKTRACE=1 ./target/debug/ln < test

