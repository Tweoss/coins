cd text_cli;
cargo run 1000;
cd ../view;
open http://localhost:4000/;
basic-http-server;
open ../text_cli;