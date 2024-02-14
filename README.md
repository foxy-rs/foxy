# foxy-rs

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/R6R8PGIU6)

```rust
use foxy::prelude::*;
use tracing::debug;

pub struct App;

impl Runnable for App {
  fn settings() -> FoxyCreateInfo {
    FoxyCreateInfo::default()
      .with_debug_info(DebugInfo::Shown)
      .with_polling(Polling::Poll)
  }

  fn new(_foxy: &Foxy) -> Self {
    Self {}
  }

  fn update(&mut self, _foxy: &Foxy, event: &FoxyEvent) {
    if let FoxyEvent::Input(InputEvent::Keyboard(..)) = event {
      debug!("UPDATE: {:?}", event)
    }
  }
}

fn main() -> FoxyResult<()> {
  start_debug_logging_session!();

  App::run()
}
```

## `foxy` is a simple engine backbone and graphics renderer

The main goal of `foxy` is to be a simple, easy-to-use API. ⚠️ This project is still very much a WIP; I am only one student, after all. ⚠️ While high-performance is obviously a secondary goal, ease of implementation, with regards to the internal framework and the external API, are primary. It will be using Vulkan to allow for hardware raytracing.

There are **2** primary threads in `foxy`:

* **main:** where the rendering happens and the message pump lives
* **foxy:** where all the main application code is executed

This layout was chosen to allow for the window messages not to block the application, and to allow rendering not to block on the application code.

## Why so many crates?

This repository contains a few crates as they each naturally evolved and split apart:

* **foxy:** a simple app framework.
* **foxy_window:** a simplified, Rust-y API for creating a window using Win32.
* **foxy_renderer:** a simplified, Rust-y API for drawing to a canvas.
* **foxy_utils:** a small utilties library.

## Thanks to

* Piston: for the idea of how a simple, Rust-y API for an engine might look light.
* Winit: as a reference on events and how to best structure them.
* Vulkan Guide (<https://vkguide.dev/>): for being a wealth of knowledge on architecturing a vulkan application.
* Brenden Galea: for his amazing Vulkan tutorial series (which I would love for him to continue!).
* One Lone Coder: for inspiration in simplicity and ease of use.
* GetIntoGameDev: for his outstanding Vulkan tutorials in C++.

WGPU:

* <https://whoisryosuke.com/blog/2022/render-pipelines-in-wgpu-and-rust>
* <https://sotrh.github.io/learn-wgpu/>
