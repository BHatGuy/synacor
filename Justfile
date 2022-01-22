teleport:
    cargo run --bin teleport --release
default := 'challenge.bin'
run file='challenge.bin':
    cargo run --release {{file}}
debug:
    socat - /tmp/synacor.sock