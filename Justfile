#!/usr/bin/env just --justfile

check:
    cargo check
    cargo test

lint:
    cargo clippy

test-codecrafters:
    codecrafters test

test-all-codecrafters:
    codecrafters test -previous

submit-codecrafters:
    codecrafters submit
