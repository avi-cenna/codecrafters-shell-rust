#!/usr/bin/env just --justfile

check:
    cargo check
    cargo test

lint:
    cargo clippy

test-codecrafters *args:
    codecrafters test {{ args }}

submit-codecrafters:
    codecrafters submit
