cd text_cli;
cargo run;
cat rendered_dump.json | pbcopy;
cd ../view;
open http://localhost:4000/;
basic-http-server 