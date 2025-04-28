# ODDOR Truckshift for Linux

Userspace driver for [this USB truck shifter][amazon-oddor-truckshift] to be usable in Euro Truck Simulator 2.

It is possible that this driver could work for other brands which use the same hardware underneath, but I obviously have
no way of testing this. If you do find others that work, feel free to create an issue so we can add it here and let
people know.

Below is the TL;DR setup, if you want to read some more background story, it is available [here][blogpost].

There is no official release at the moment, but if there is enough interest in this, I'd be willing to create an AUR
package.

## TL;DR Setup

1. Create `/etc/udev/rules.d/99-oddor-truckshift.rules` with the following content:

```
SUBSYSTEM=="usb", ATTRS{idVendor}=="1020", ATTRS{idProduct}=="8863", GROUP="users", MODE="0660"
KERNEL=="event*", SUBSYSTEM=="input", ATTRS{id/vendor}=="1020", ATTRS{id/product}=="8863", GROUP="users", MODE="0660" SYMLINK+="oddor_truckshift" RUN+="/bin/chmod 0660 /dev/oddor_truckshift"
```

2. Clone this repo and run `cargo build --release` to create the `oddor_truckshift` executable in the `target/release/`
   dir.

From then on, you can just run this executable before staring a game, and the shifter will be visible in your game as a
`libinput` device with three buttons:

* `MODE` - Front switch, representing the range selector.
* `GEAR_UP` - Side switch, representing the gear splitter.
* `EXTRA` - Round button at the top of the shifter. I personally use this one as a handbrake/parking brake, but
  it's obviously a free-for-all.

## Additional Info

If you get an error message with this text:

```
Warning: your libusb doesn't support hotplug.
Fatal error: No matching USB devices found.
```

...it means two things: you have a very old version of libusb which doesn't have support for hotplug events, and your
shifter is physically not plugged in. In this case, you just need to plug in the shifter before running the executable,
and everything should be fine.

If your shifter _is_ already plugged in, then there is probably mismatch in the vendor/product ID, which you can check
with `lsusb`. If there is a mismatch, it could be a different controller that won't work with this, but it could also be
a different ID combination with the same hardware underneath. The only way to find out is to try it yourself by changing
the constants in `constants.rs` and rebuilding.

If you don't want to start this driver every time you want to play a game, and your `libusb` supports hotplug, you can
create a simple systemd unit and let it run in the background. It takes only ~5MB of RAM, and is just sitting idle until
you plug in your shifter.

## Known Issues

* In some cases, like quickloading a game in ETS2, the game will reset its internal states of all the switches. In
  practical terms, this means you'll need to flip the switch twice in order to let the game know the current state. For
  example, if you're in 3rd gear high, meaning your `GEAR_UP` switch is up, and you load a game, it will assume that the
  initial state is down. Basically, the physical state of the switches becomes desynchronised with the game states. In
  theory this could be solved by having the driver send the states periodically, but this was a minor inconvenience so
  far, so I opted not to do it.

[amazon-oddor-truckshift]: https://www.amazon.de/-/en/gp/product/B09C4YKB2B
[blogpost]: http://dsimidzija.github.io/posts/euro-truck-simulator-2-oddor-truckshifter-linux/
