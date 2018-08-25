# rpi-music-visualizer
Raspberry Pi Music Visualizer written in Rust

## Roadmap

[x] Accept microphone input

[x] Plug in LEDs

[x] LED communication

[x] Battery setup

[x] Wiring diagram

[x] Microphone

[ ] How to wear it (Comfortably)

[ ] Improve visualizer data quality

[ ] More visualizers

# How to run on Raspberry Pi

## Build

cargo build --release --features hardware

## Run

./rpi.sh --screen hardware

## Alsa configuration
Using a USB microphone on the Raspberry Pi.

```
pcm.microphone {
  type dsnoop
  ipc_key 1
  slave {
    pcm "hw:2,0"

    period_size 1024
    buffer_size 1024
    rate 24000
    periods 0
    period_time 0
  }
}

pcm.!default {
  type route

  slave.pcm microphone
  slave.channels 2

  ttable {
    # Copy both input channels to output channel 0 (Left).
    0.0 0.5
    1.0 0.5
    # Send nothing to output channel 1 (Right).
    0.1 0
    1.1 0
  }
}

ctl.!default {
  type hw
  card 2
}
```
