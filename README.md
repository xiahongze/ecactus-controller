# ecactus-controller

## Overview

The `ecactus-controller` is a Rust-based application designed to manage and control the eCactus system. It interacts
with the eCactus API to set and retrieve charge modes, device information, and run data.

The main enhancement is that it allows a more complex scheduled charging scheme:

- **Conservative**: set a preferred SoC and a duration. This mode basically prefers charging the battery for a certain
  time to reach the preferred SoC. After that, it will revert to the default mode (self-sufficient).
- **Active**: set a charging power and a duration. This is helpful to me because I have two independent PV systems,
  one of which is connected to the battery and one is not. However, to use the power from the other system more
  efficiently, I can set a higher charging power for a certain time. It works for me because the grid charges me
  the net export/import power. Update `compute_charge_power` in `src/state.rs` to fit your needs.
- **Self-sufficient**: the default mode. The battery will be charged by the PV system and discharged to the house
  according to the house consumption.

## Development

To build the project, ensure you have Rust and Cargo installed. Then, navigate to the project directory and run:

```sh
cargo build --release
```

The binary will be located at `target/release/ecactus_controller`.

To run the project, use:

```sh
cargo run
```

To run the tests, use:

```sh
cargo test
```

## How to Run

I recommend running the application as a systemd service. For me, I set up the service on a Raspberry Pi.
`ecactus-controller.service` is provided as an example.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

Use at your own risk. I am not responsible for any damage caused by using this software.
