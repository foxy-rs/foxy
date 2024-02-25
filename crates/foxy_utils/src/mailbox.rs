use crossbeam::channel::{Receiver, RecvError, SendError, Sender, TryRecvError};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Mailbox<SenderMessage: Send + Sync, ReceiverMessage: Send + Sync> {
  sender: Sender<SenderMessage>,
  receiver: Receiver<ReceiverMessage>,
}

impl<MessageA: Send + Sync, MessageB: Send + Sync> Mailbox<MessageA, MessageB> {
  pub fn new_entangled_pair() -> (Mailbox<MessageA, MessageB>, Mailbox<MessageB, MessageA>) {
    let (sender_a, receiver_a) = crossbeam::channel::unbounded();
    let (sender_b, receiver_b) = crossbeam::channel::unbounded();

    let mailbox_a = Mailbox::new(sender_a, receiver_b);
    let mailbox_b = Mailbox::new(sender_b, receiver_a);

    (mailbox_a, mailbox_b)
  }
}

impl<SenderMessage: Send + Sync, ReceiverMessage: Send + Sync> Mailbox<SenderMessage, ReceiverMessage> {
  pub fn new(sender: Sender<SenderMessage>, receiver: Receiver<ReceiverMessage>) -> Self {
    Self { sender, receiver }
  }

  pub fn send(&self, message: SenderMessage) -> Result<(), MessagingError<SenderMessage>> {
    self.sender.send(message).map_err(MessagingError::from)
  }

  pub fn recv(&self) -> Result<ReceiverMessage, MessagingError<SenderMessage>> {
    self.receiver.recv().map_err(MessagingError::from)
  }

  pub fn try_recv(&self) -> Result<ReceiverMessage, MessagingError<SenderMessage>> {
    self.receiver.try_recv().map_err(MessagingError::from)
  }

  pub fn send_and_recv(&self, message: SenderMessage) -> Result<ReceiverMessage, MessagingError<SenderMessage>> {
    self.send(message)?;
    self.recv()
  }
}

#[derive(Error, Debug)]
pub enum MessagingError<SenderMessage> {
  #[error("failed to send message")]
  SendError {
    #[from]
    error: SendError<SenderMessage>,
  },
  #[error("{error}")]
  TryRecvError {
    #[from]
    error: TryRecvError,
  },
  #[error("{error}")]
  RecvError {
    #[from]
    error: RecvError,
  },
}
