#!/bin/bash

if [[ -z $1 ]]; then
	echo "testing workspace"
	cargo test --workspace && cargo test --workspace --all-features
else
	echo "testing $1"
	cargo test -p $1 && cargo test -p $1 --all-features
fi
