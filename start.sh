#!/bin/bash


if [ -z "$1" ]; then
	echo "no network passed, assuming REGTEST"
	export NETWORK=regtest
else
	export NETWORK=$1
fi

export RUST_BACKTRACE=full

#create new plugin log file
rm /tmp/pluginlog
touch /tmp/pluginlog
export PLUGIN=$PWD/target/debug/sweep-plugin

# start bitcoind 
bitcoind -$NETWORK -txindex --daemon
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
lightningd --daemon --network=$NETWORK --lightning-dir=/tmp/l1
lightningd --daemon --network=$NETWORK --lightning-dir=/tmp/l2 --bind-addr=/tmp/l2/peer --plugin=$PLUGIN

function bcli () {
	bitcoin-cli -$NETWORK $@
}

function l1 () {
	lightning-cli --network=$NETWORK --lightning-dir=/tmp/l1 $@
}

function l2 () {
	lightning-cli --network=$NETWORK --lightning-dir=/tmp/l2 $@
}

