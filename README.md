# ODDOR H shift for Linux

Userspace driver for ODDOR H-SHIFT gearbox to be usable in Assetto Corsa.

It is possible that this driver could work for other brands which use the same hardware underneath, but I obviously have
no way of testing this. If you do find others that work, feel free to create an issue so we can add it here and let
people know.

Below is the TL;DR setup, if you want to read some more background story, it is available [here][blogpost].

There is no official release at the moment, but if there is enough interest in this, I'd be willing to create an AUR
package.

## TL;DR Setup

1. Create `/etc/udev/rules.d/99-oddor-h-shift.rules` with the following content:

```
SUBSYSTEM=="usb", ATTRS{idVendor}=="4785", ATTRS{idProduct}=="7353", GROUP="users", MODE="0660"
KERNEL=="event*", SUBSYSTEM=="input", ATTRS{id/vendor}=="4785", ATTRS{id/product}=="7353", GROUP="users", MODE="0660" SYMLINK+="oddor_h_shift" RUN+="/bin/chmod 0660 /dev/oddor_h_shift"
```

2. Clone this repo and run `cargo build --release` to create the `oddor_h_shift` executable in the `target/release/`
   dir.

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

[blogpost]: http://dsimidzija.github.io/posts/euro-truck-simulator-2-oddor-truckshifter-linux/
