#!/usr/bin/env just --justfile

check:
    cargo fmt
    cargo check
    cargo test

lint:
    cargo fmt
    cargo clippy

test-codecrafters:
    codecrafters test

test-all-codecrafters:
    codecrafters test -previous

submit-codecrafters:
    codecrafters submit
