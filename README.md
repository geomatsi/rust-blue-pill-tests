![Rust](https://github.com/geomatsi/rust-blue-pill-tests/workflows/Rust/badge.svg?branch=master)

# cargo-make tools
Start tmux debug environment with ST-Link:
```bash
$ cargo make debug
```

# Debug options
## Semihosting debug
Commands:
```bash
  $ sudo openocd -f tools/openocd.cfg -c 'attach ()'
  $ cargo build --bin test
  $ cargo run --bin test
```

## ITM debug
Commands:
```bash
  $ mkfifo /tmp/itm.fifo
  $ ~/.cargo/bin/itmdump -f /tmp/itm.fifo -F
  $ cargo build --bin test
  $ cargo run --bin test
```

SWO pin PB3 on BluePill has to be connected to appropriate SWO pin on
debugger/programmer. Note that ST-Link does not have SWO pin,
while Jlink Pro has.
