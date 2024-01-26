[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/R6R8PGIU6)

```rust
use foxy::prelude::*;
use tracing::*;

fn main() {
  start_debug_logging_session!();

  let foxy = FoxyLifecycle::builder()
    .with_title("Simple")
    .with_size(800, 450)
    .build_unwrap();

  for stage in foxy {
    match stage {
      Stage::FixedUpdate { .. } => debug!("FixedUpdate"),
      Stage::Update { .. } => debug!("Update"),
      _ => {}
    }
  }
}
```
## `foxy` is a simple engine backbone and graphics renderer.
This project is still very much a WIP. It will be using Vulkan to allow for hardware raytracing. The window uses `foxy_window` to create a Win32 desktop window. 

There are **3** primary threads in `foxy`:

* **main:** where all the main application code is executed
* **window:** where the window and window message pump live
* **render:** where the rendering happens

This layout was chosen to allow for the window messages not to block the application, and to allow rendering not to block on the application code.

## Why so many crates?

This repository contains a few crates as they each naturally evolved and split apart:

* **foxy:** a simple renderer which is a WIP. Still deciding on OpenGL or Vulkan. OpenGL has benefits with quick development, but Vulkan would allow for hardware raytracing.
* **foxy_window:** a simplified, Rust-y API for creating a window using Win32.
* **foxy_vulkan:** a simplified, Rust-y API for using Vulkan.
* **foxy_renderer:** a simplified, Rust-y API for drawing to a canvas.
* **foxy_util:** a small utilties library.
* **foxy_types:** a placeholder library.

## Thanks to:
* Piston: for the idea of how a simple, Rust-y API for an engine might look light.
* Winit: as a reference on events and how to best structure them.
* GetIntoGameDev: for his outstanding Vulkan tutorials in C++.
