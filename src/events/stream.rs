//! Event stream.

use futures::Stream;
use heapless::Deque;

use crate::events::{self, Event};
use crate::io;

#[cfg(test)]
mod tests;

#[derive(Debug)]
struct Context<'a, ReaderTy> {
    reader: &'a mut ReaderTy,
    buffer: [u8; 32],
    rpos: usize,
    wpos: usize,
    queue: Deque<io::Result<Event>, 32>,
}

impl<'a, ReaderTy> Context<'a, ReaderTy> {
    fn new(reader: &'a mut ReaderTy) -> Self {
        Context {
            reader,
            buffer: Default::default(),
            rpos: 0,
            wpos: 0,
            queue: Default::default(),
        }
    }
}

pub fn stream<ReaderTy>(reader: &mut ReaderTy) -> impl Stream<Item = io::Result<Event>>
where
    ReaderTy: io::Read + Send,
{
    futures::stream::unfold(Context::new(reader), |mut cx| async move {
        loop {
            // We start by purging the queue of pending events in order to preserve causality.
            if let Some(event) = cx.queue.pop_front() {
                return Some((event, cx));
            }

            let mut byte = [0u8; 1];

            let nbytes = match cx.reader.read(&mut byte).await {
                Ok(n) => n,
                Err(err) => return Some((Err(err), Context::new(cx.reader))),
            };

            if 0 == nbytes {
                return None;
            }

            cx.buffer[cx.wpos] = byte[0];
            cx.wpos += 1;

            let Ok(mut input) = str::from_utf8(&cx.buffer[cx.rpos..cx.wpos]) else {
                return Some((Err(io::Error::Unknown), Context::new(cx.reader)));
            };

            loop {
                if input.is_empty() {
                    cx.rpos = 0;
                    cx.wpos = 0;
                    break;
                }

                match events::parse(input) {
                    Ok((rest, event)) => {
                        cx.rpos += input.len() - rest.len();
                        input = rest;

                        if cx.queue.is_full() {
                            cx.queue.pop_front();
                        }

                        // SAFETY: we can ensure that the queue is not full thanks to the lines
                        // above.
                        unsafe { cx.queue.push_back_unchecked(Ok(event)) }
                    }

                    Err(nom::Err::Incomplete(_)) => break,
                    Err(nom::Err::Error(_)) => break,

                    // Unrecoverable error, then reset the context.
                    Err(nom::Err::Failure(_)) => {
                        if cx.queue.is_full() {
                            cx.queue.pop_front();
                        }

                        unsafe { cx.queue.push_back_unchecked(Err(io::Error::Unknown)) };

                        cx.rpos = 0;
                        cx.wpos = 0;
                        break;
                    }
                }
            }
        }
    })
}
