This repository contains 4 crates:
* **ookami:** a simple renderer which is a WIP. Still deciding on OpenGL or Vulkan. OpenGL has benefits with quick development, but Vulkan would allow for hardware raytracing.
* **ezwin:** a simplified API for using Win32 to create a window and have it hidden behind a Rust-y API.
* **eztracing:** a small, lightweight wrapper around `tracing` which just makes it easier for me to enable or disable logging as I usually do in most of my projects.
* **messaging:** an easy two-way messaging crate. Good for use cases such as communication across two threads.
