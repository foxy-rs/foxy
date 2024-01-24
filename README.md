## Why so many crates?

This repository contains a few crates as they each naturally evolved and split apart:

* **ookami:** a simple renderer which is a WIP. Still deciding on OpenGL or Vulkan. OpenGL has benefits with quick development, but Vulkan would allow for hardware raytracing.
* **foxy_window:** a simplified, Rust-y API for creating a window using Win32.
* **foxy_vulkan:** a simplified, Rust-y API for using Vulkan.
* **foxy_renderer:** a simplified, Rust-y API for drawing to a canvas.
* **eztracing:** a small, lightweight wrapper around `tracing` which just makes it easier for me to enable or disable logging as I usually do in most of my projects.

# Ookami

`ookami` is a simple renderer which is a WIP. Still deciding on OpenGL or Vulkan. OpenGL has benefits with quick development, but Vulkan would allow for hardware raytracing. The window uses `ezwin` to create a Win32 desktop window. 

There are **3** primary threads in `ookami`:

* **main:** where all the main application code is executed
* **window:** where the Win32 message pump lives
* **render:** where the rendering happens

This layout was chosen to allow for the window messages not to block the application, and to allow rendering not to block on the application code.
