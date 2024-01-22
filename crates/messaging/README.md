 # A simple two-way messaging crate
 [![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/R6R8PGIU6)
 
Good for use cases such as communication across two threads.

```rust
let (renderer_mailbox, game_mailbox) = Mailbox::new_entangled_pair();

renderer_mailbox.send(RenderLoopMessage::SyncWithGame);
if let Ok(RenderLoopMessage::SyncWithGame) = game_mailbox.poll() {
    // ...
}

game_mailbox.send_and_wait(GameLoopMessage::SyncWithRender)?;
```
