#!/bin/bash

sudo cargo build --release
cp -rf target/release/encrypt-key ./


encrypt-key --decode --iv=IkaxEcg9j2BMoume --key=AZpHxxUhOHtEd+J2v5G/YWI4Wg+FXae+HacSw3kjtY3+sugQs+syC/uV/AYmI/7bx7zHke93YJ5tEOu5fYhH8Ek+xww4gwy1ICoMpH2T+Y8AEXG+2Pu14G8XDT3aj4uUoG0xnJI+e/A= --pwd=
