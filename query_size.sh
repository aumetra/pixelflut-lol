#!/usr/bin/env bash

set -eou pipefail

if [ -z "$1" ] || [ -z "$2" ]; then
	echo "Usage: $0 [IP/hostname of server] [Port of server]"
	exit 1
fi

echo -n "SIZE\n" | nc $1 $2

