#!/bin/bash

sudo cargo build --release
cp -rf target/release/sol-safekey ./
