#!/usr/local/bin/fish
# convenience shell script. uses a cargo binary.
cd ../text_cli;
cargo run 1000;
cd ../view;
basic-http-server -a 0.0.0.0:4000 &;
cp ../text_cli/rendered_dump.cbor dump.cbor
open http://localhost:4000/;
open ../text_cli;
