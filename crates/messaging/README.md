 # An easy two-way messaging crate. Good for use cases such as communication across two threads.


Example
```rust
let (renderer_mailbox, game_mailbox) = Mailbox::new_entangled_pair();

renderer_mailbox.send_and_wait(RenderLoopMessage::SyncWithGame);
if let Ok(RenderLoopMessage::SyncWithGame) = game_mailbox.poll() {
    // ...
}
game_mailbox.send_and_wait(GameLoopMessage::SyncWithRender);
if let Ok(GameLoopMessage::SyncWithRender) = renderer_mailbox.poll() {
    // ...
}
```
