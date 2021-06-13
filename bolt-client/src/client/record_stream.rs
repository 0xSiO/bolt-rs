use std::{
    collections::VecDeque,
    pin::Pin,
    task::{Context, Poll},
};

use bolt_proto::{message::Record, Message};
use futures_util::{
    future::BoxFuture,
    io::{AsyncRead, AsyncWrite},
    ready,
    stream::Stream,
};

use crate::{error::*, Client};

struct RecordStream<'c, 'f, S: AsyncRead + AsyncWrite + Unpin + Send> {
    client: &'c mut Client<S>,
    record_buffer: VecDeque<Record>,
    summary_buffer: Option<Message>,
    read_future: Option<BoxFuture<'f, Result<Message>>>,
}

// TODO: The way this is written, the server can send a summary message that is not in the allowed
//       summary message types. Maybe we should return an error if this happens?
impl<'c: 'f, 'f, S: AsyncRead + AsyncWrite + Unpin + Send> RecordStream<'c, 'f, S> {
    pub(crate) fn new(client: &'c mut Client<S>) -> Self {
        Self {
            client,
            record_buffer: Default::default(),
            summary_buffer: None,
            read_future: Default::default(),
        }
    }

    pub async fn summary(&mut self) -> Result<Message> {
        if let Some(summary) = self.summary_buffer.take() {
            return Ok(summary);
        }

        loop {
            let message = if self.read_future.is_some() {
                self.read_future.take().as_mut().unwrap().await?
            } else {
                self.client.read_message().await?
            };

            match message {
                Message::Record(record) => self.record_buffer.push_back(record),
                other => return Ok(other),
            }
        }
    }

    fn poll_read(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<Record>>> {
        let result = ready!(self.read_future.as_mut().unwrap().as_mut().poll(cx));
        // Remove the future if it finished
        self.read_future.take();

        match result {
            Ok(Message::Record(record)) => Poll::Ready(Some(Ok(record))),
            Ok(message) => {
                // A non-RECORD message indicates the end of the result stream.
                self.summary_buffer.replace(message);
                Poll::Ready(None)
            }
            Err(error) => Poll::Ready(Some(Err(error))),
        }
    }
}

impl<'c: 'f, 'f, S: AsyncRead + AsyncWrite + Unpin + Send> Stream for RecordStream<'c, 'f, S> {
    type Item = Result<Record>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(record) = self.record_buffer.pop_front() {
            return Poll::Ready(Some(Ok(record)));
        }

        if self.read_future.is_some() {
            return self.poll_read(cx);
        }

        // TODO: self is dropped at the end of this function, so read_future cannot hold a
        //       reference to it.
        // self.read_future = Some(Box::pin(self.client.read_message()));

        ready!(self.poll_read(cx));

        todo!()
        // Poll::Pending
    }
}
