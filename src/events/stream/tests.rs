use futures::{StreamExt, pin_mut};
use speculoos::prelude::*;

use crate::events::{self, Event, KeyCode, KeyEvent};

impl crate::io::Read for String {
    async fn read(&mut self, data: &mut [u8]) -> crate::io::Result<usize> {
        let n = self.len().min(data.len());
        let (input, _) = self.as_bytes().split_at(n);
        let (output, _) = data.split_at_mut(n);
        output.copy_from_slice(input);
        Ok(n)
    }
}

#[tokio::test]
async fn it_should_stream_empty_events() {
    let mut input = String::from("");

    let stream = events::stream(&mut input);
    pin_mut!(stream);

    assert_that!(stream.next().await).is_none();
}

#[tokio::test]
async fn it_should_stream_single_key_event() {
    let mut input = String::from("\x0d");

    let stream = events::stream(&mut input);
    pin_mut!(stream);

    assert_that!(stream.next().await)
        .is_some()
        .is_ok()
        .is_equal_to(Event::Key(KeyEvent::from(KeyCode::Enter)));
}
