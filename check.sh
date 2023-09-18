#!/bin/bash

if [[ -z $1 ]]; then
	echo "checking workspace"
	cargo check --workspace --tests && cargo check --workspace --tests --all-features
else
	echo "checking $1"
	cargo check -p $1 --tests && cargo check -p $1 --tests --all-features
fi
