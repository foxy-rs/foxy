[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/R6R8PGIU6)

```rust
use foxy::prelude::*;
use tracing::debug;

fn main() {
  if cfg!(debug_assertions) {
    logging_session!().start();
  }

  let mut app = App::builder()
    .with_title("Foxy")
    .with_size(800, 450)
    .build()
    .unwrap_or_else(|e| panic!("{e}"));

  while let Some(message) = app.wait() {
    match message {
      Lifecycle::Entering => debug!("Entering"),
      Lifecycle::Update { .. } => debug!("Update"),
      Lifecycle::Exiting => debug!("Exiting"),
      _ => {}
    }
  }
}
```
## `foxy` is a simple engine skeleton and graphics renderer.
This project is still very much a WIP. I am still deciding on OpenGL or Vulkan. OpenGL has benefits with quick development, but Vulkan would allow for hardware raytracing. The window uses `foxy_window` to create a Win32 desktop window. 

There are **3** primary threads in `foxy`:

* **main:** where all the main application code is executed
* **window:** where the Win32 message pump lives
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
