#!/usr/bin/env bash

set -o nounset -o pipefail -o errexit

DIR="$(cd "$( dirname "${BASH_SOURCE[0]}")" && pwd)"
DATADIR="$DIR/../data"

CC_DBS="${CC_DBS:-superheroes companies}"
CC_GIT=https://raw.githubusercontent.com/codecrafters-io/sample-sqlite-databases/master

download() {
    echo "Downloading $CC_DBS"

    for db in $CC_DBS; do
        curl -Lo "$DATADIR/$db.db" "$CC_GIT/$db.db"
    done
}

download "$@"
