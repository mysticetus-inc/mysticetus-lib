use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures::Stream;

use crate::error::{Error, InvalidEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Put,
    Patch,
    KeepAlive,
    Cancel,
    AuthRevoked,
}

impl EventType {
    pub const fn as_str(&self) -> &'static str {
        match *self {
            Self::Put => "put",
            Self::Patch => "patch",
            Self::KeepAlive => "keep-alive",
            Self::Cancel => "cancel",
            Self::AuthRevoked => "auth_revoked",
        }
    }
}

impl fmt::Display for EventType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub event_type: EventType,
    pub payload: Option<EventPayload>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct EventPayload {
    path: String,
    data: serde_json::Value,
}

impl Event {
    fn parse_bytes(mut bytes: Bytes) -> Result<Self, Error> {
        let newline_pos = bytes
            .iter()
            .position(|c| *c == b'\n')
            .ok_or(InvalidEvent::MissingEvent)?;

        let event_type = match std::str::from_utf8(bytes.get(..newline_pos).unwrap())? {
            "event: put" => EventType::Put,
            "event: patch" => EventType::Patch,
            "event: keep-alive" => EventType::KeepAlive,
            "event: cancel" => EventType::Cancel,
            "event: auth_revoked" => EventType::AuthRevoked,
            unknown => return Err(InvalidEvent::unknown_event_type(unknown).into()),
        };

        let data_line_start = newline_pos + 1;

        let data_tag_slice = bytes
            .get(data_line_start..(data_line_start + 6))
            .ok_or_else(|| InvalidEvent::missing_data(event_type))?;

        let data_bytes = match std::str::from_utf8(data_tag_slice)? {
            "data: " => bytes.split_off(data_line_start + 6),
            unknown => {
                println!("'{unknown}'");
                return Err(InvalidEvent::missing_data(event_type).into());
            }
        };

        print!("raw data: '{}'", String::from_utf8_lossy(&data_bytes));

        let payload =
            serde_json::from_slice(&*data_bytes).map_err(InvalidEvent::InvalidDataPayload)?;

        Ok(Self {
            event_type,
            payload,
        })
    }
}

pub struct EventStream {
    stream: Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>>>>,
}

impl EventStream {
    pub(crate) fn from_response(resp: reqwest::Response) -> Self {
        Self {
            stream: Box::pin(resp.bytes_stream()),
        }
    }
}

impl Stream for EventStream {
    type Item = Result<Event, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.stream.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => Poll::Ready(Some(Event::parse_bytes(bytes))),
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(err.into()))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
