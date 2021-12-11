# Example

This is not meant for pedagogical purposes, more as a reference for myself when
doing that in other places.

Building and running:

1. Make sure you have all system dependencies, then `cargo run`, or
2. With direnv, the dependencies should take care of themselves. Just `cargo run`.

Reproducible builds with `nix build` are WIP, but getting all the vulkan deps
right is challenging.

## Improvements

We could render multiple frames at once. The way I'd do that by defining a
batch size (say _n_), having _n_ textures and a buffer with _n_ * `FRAME_SIZE`
capacity. Then in each render pass, render a frame to each texture and copy to
the buffer at the proper offset (_i_ * `FRAME_SIZE`) (see the
`copy_texture_to_buffer()` call in the soruce).

As it is, though, it is fast enough (a 12 seconds video is rendered and encoded
in ~4 seconds in debug mode in my laptop). Further optimization is left to
later.
