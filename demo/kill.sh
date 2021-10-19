#!/usr/local/bin/fish
kill (ps | grep "basic-http-server" | grep -v "grep" | sed -E 's/^([^ ]+).*$/\1/'); and echo "killed basic server";
kill (ps | grep -E "target/debug/coins 0.\d+ 0.\d+ 0.\d+" | grep -v "grep" | sed -E 's/^([^ ]+).*$/\1/'); and echo "killed player server";
