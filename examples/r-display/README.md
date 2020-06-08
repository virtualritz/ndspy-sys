# r-display

Minimal [NSI](https://nsi.readthedocs.io/)/RenderMan 8bit PNG display driver template written in Rust.

The build only works with [3Delight](https://www.3delight.com/) out of the box. Build instructions should work for **Linux** and **macOS**. On **Windows** your mileage my vary.

The [`ndspy-sys`](https://github.com/virtualritz/r-display/blob/master/ndspy-sys/) crate which is part of this project uses the `$DELIGHT` environment variable to find the needed display driver API headers. Edit [`ndspy-sys/build.rs`](https://github.com/virtualritz/r-display/blob/master/ndspy-sys/build.rs) to add (an) additional or different search path(s) for these headers.
