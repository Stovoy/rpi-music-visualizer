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
pcm.!default {
  type dsnoop
  ipc_key 1
  slave {
    pcm "hw:1,0"
    channels 1

    period_size 1024
    buffer_size 24000
    rate 24000
    periods 0
    period_time 0
  }
}

ctl.!default {
  type hw
  card 0
}
```
