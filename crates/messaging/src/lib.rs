use std::sync::mpsc::{Receiver, RecvError, SendError, Sender, TryRecvError};

///
/// An object that has the ability to act as a two-way channel. This allows for two-way communications across threads, for example.
///
#[derive(Debug)]
pub struct Mailbox<SenderMessage: Send + Sync, ReceiverMessage: Send + Sync> {
    sender: Sender<SenderMessage>,
    receiver: Receiver<ReceiverMessage>,
}

impl<MessageA: Send + Sync + 'static, MessageB: Send + Sync + 'static> Mailbox<MessageA, MessageB> {
    ///
    /// Creates a new pair of `Mailbox`s in the form of `(Mailbox<MessageA, MessageB>, Mailbox<MessageB, MessageA>)`
    ///
    /// # Example
    /// ```rust
    /// let (renderer_mailbox, game_mailbox) = Mailbox::new_entangled_pair();
    ///
    /// renderer_mailbox.send(RenderLoopMessage::SyncWithGame);
    /// if let Ok(RenderLoopMessage::SyncWithGame) = game_mailbox.poll() {
    ///     // ...
    /// }
    /// 
    /// game_mailbox.send_and_wait(GameLoopMessage::SyncWithRender)?;
    /// ```
    ///
    pub fn new_entangled_pair() -> (Mailbox<MessageA, MessageB>, Mailbox<MessageB, MessageA>) {
        let (sender_a, receiver_a) = std::sync::mpsc::channel();
        let (sender_b, receiver_b) = std::sync::mpsc::channel();

        let mailbox_a = Mailbox::new(sender_a, receiver_b);
        let mailbox_b = Mailbox::new(sender_b, receiver_a);

        (mailbox_a, mailbox_b)
    }
}

impl<SenderMessage: Send + Sync, ReceiverMessage: Send + Sync>
    Mailbox<SenderMessage, ReceiverMessage>
{
    ///
    /// Creates a new `Mailbox`. Since `new_entangled_pair()` exists, this is only exposed in case that doesn't cover your use case. Otherwise you should just use that method instead.
    ///
    /// # Example
    /// ```rust
    /// let (sender_a, receiver_a) = std::sync::mpsc::channel();
    /// let (sender_b, receiver_b) = std::sync::mpsc::channel();
    ///
    /// let renderer_mailbox = Mailbox::new(sender_a, receiver_b);
    /// let game_mailbox = Mailbox::new(sender_b, receiver_a);
    ///
    /// renderer_mailbox.send(RenderLoopMessage::SyncWithGame);
    /// if let Ok(RenderLoopMessage::SyncWithGame) = game_mailbox.poll() {
    ///     // ...
    /// }
    /// 
    /// game_mailbox.send_and_wait(GameLoopMessage::SyncWithRender)?;
    /// ```
    ///
    pub fn new(sender: Sender<SenderMessage>, receiver: Receiver<ReceiverMessage>) -> Self {
        Self { sender, receiver }
    }

    ///
    /// Sends a message through the held sender. Wrapper around `sender.send()`
    /// 
    pub fn send(&self, message: SenderMessage) -> Result<(), MessagingError<SenderMessage>> {
        self.sender.send(message).map_err(MessagingError::from)
    }

    ///
    /// Waits for a message from the held receiver. Wrapper around `receiver.recv()`
    /// 
    pub fn wait(&mut self) -> Result<ReceiverMessage, MessagingError<SenderMessage>> {
        self.receiver.recv().map_err(MessagingError::from)
    }

    ///
    /// Polls for a message from the held receiver, otherwise immediately returning. Wrapper around `receiver.try_recv()`
    /// 
    pub fn poll(&mut self) -> Result<ReceiverMessage, MessagingError<SenderMessage>> {
        self.receiver.try_recv().map_err(MessagingError::from)
    }

    ///
    /// Does a `send` and a `wait`. Good for syncing between two threads.
    /// 
    pub fn send_and_wait(&mut self, message: SenderMessage) -> Result<ReceiverMessage, MessagingError<SenderMessage>> {
        self.send(message)?;
        self.wait()
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MessagingError<SenderMessage> {
    #[error("{error}")]
    SendError { #[from] error: SendError<SenderMessage> },
    #[error("{error}")]
    PollError { #[from] error: TryRecvError },
    #[error("{error}")]
    WaitError { #[from] error: RecvError },
}
