#!/usr/local/bin/fish
# Starting the server
cd ../server; cargo run 0.0 0.0 0.0;
# Flushing the data
./flush.sh
# Processing, opening the viewer
./run.sh
# Killing the basic and player server
./kill.sh
