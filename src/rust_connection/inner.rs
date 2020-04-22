//! A pure-rust implementation of a connection to an X11 server.

use std::collections::VecDeque;
use std::io::Write;

use super::RawEventAndSeqNumber;
use crate::connection::{DiscardMode, RequestKind, SequenceNumber};
use crate::x11_utils::GenericEvent;

#[derive(Debug, Clone)]
pub(crate) enum PollReply {
    /// It is not clear yet what the result will be; try again.
    TryAgain,
    /// There will be no reply; polling is done.
    NoReply,
    /// Here is the result of the polling; polling is done.
    Reply(Vec<u8>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct SentRequest {
    seqno: SequenceNumber,
    discard_mode: Option<DiscardMode>,
}

#[derive(Debug)]
pub(crate) struct ConnectionInner<W>
where
    W: Write,
{
    // The underlying byte stream used for writing to the X11 server. Reading is done outside of
    // this struct (for synchronisation reasons).
    pub(crate) write: W,

    // The sequence number of the last request that was written
    last_sequence_written: SequenceNumber,
    // Sorted(!) list with information on requests that were written, but no answer received yet.
    sent_requests: VecDeque<SentRequest>,

    // The sequence number of the next reply that is expected to come in
    next_reply_expected: SequenceNumber,

    // The sequence number of the last reply/error/event that was read
    last_sequence_read: SequenceNumber,
    // Events that were read, but not yet returned to the API user
    pending_events: VecDeque<(SequenceNumber, Vec<u8>)>,
    // Replies that were read, but not yet returned to the API user
    pending_replies: VecDeque<(SequenceNumber, Vec<u8>)>,
}

impl<W> ConnectionInner<W>
where
    W: Write,
{
    /// Crate a `ConnectionInner` wrapping the given write stream.
    ///
    /// It is assumed that the connection was just established. This means that the next request
    /// that is sent will have sequence number one.
    pub(crate) fn new(write: W) -> Self {
        ConnectionInner {
            write,
            last_sequence_written: 0,
            next_reply_expected: 0,
            last_sequence_read: 0,
            sent_requests: VecDeque::new(),
            pending_events: VecDeque::new(),
            pending_replies: VecDeque::new(),
        }
    }

    /// Send a request to the X11 server.
    ///
    /// When this returns `None`, a sync with the server is necessary. Afterwards, the caller
    /// should try again.
    pub(crate) fn send_request(
        &mut self,
        kind: RequestKind,
    ) -> Option<SequenceNumber> {
        if self.next_reply_expected + SequenceNumber::from(u16::max_value())
            <= self.last_sequence_written && kind != RequestKind::HasResponse
        {
            // The caller need to call send_sync(). Otherwise, we might not be able to reconstruct
            // full sequence numbers for received packets.
            return None;
        }

        self.last_sequence_written += 1;
        let seqno = self.last_sequence_written;

        if kind == RequestKind::HasResponse {
            self.next_reply_expected = self.last_sequence_written;
        }

        let sent_request = SentRequest {
            seqno,
            discard_mode: None,
        };
        self.sent_requests.push_back(sent_request);

        Some(seqno)
    }

    /// Ignore the reply for a request that was previously sent.
    pub(crate) fn discard_reply(&mut self, seqno: SequenceNumber, mode: DiscardMode) {
        if let Some(entry) = self.sent_requests.iter_mut().find(|r| r.seqno == seqno) {
            entry.discard_mode = Some(mode);
        }
        match mode {
            DiscardMode::DiscardReplyAndError => self.pending_replies.retain(|r| r.0 != seqno),
            DiscardMode::DiscardReply => {
                if let Some(index) = self.pending_replies.iter().position(|r| r.0 == seqno) {
                    while self
                        .pending_replies
                        .get(index)
                        .filter(|r| r.0 == seqno)
                        .is_some()
                    {
                        if let Some((_, packet)) = self.pending_replies.remove(index) {
                            if packet[0] == 0 {
                                // This is an error
                                self.pending_events.push_back((seqno, packet));
                            }
                        }
                    }
                }
            }
        }
    }

    // Extract the sequence number from a packet read from the X11 server. The packet must be a
    // reply, an event, or an error. All of these have a u16 sequence number in bytes 2 and 3...
    // except for KeymapNotify events.
    fn extract_sequence_number(&mut self, buffer: &[u8]) -> Option<SequenceNumber> {
        use crate::xproto::KEYMAP_NOTIFY_EVENT;
        if buffer[0] == KEYMAP_NOTIFY_EVENT {
            return None;
        }
        // We get the u16 from the wire...
        let number = u16::from_ne_bytes([buffer[2], buffer[3]]);

        // ...and use our state to reconstruct the high bytes
        let high_bytes = self.last_sequence_read & !SequenceNumber::from(u16::max_value());
        let mut full_number = SequenceNumber::from(number) | high_bytes;
        if full_number < self.last_sequence_read {
            full_number += SequenceNumber::from(u16::max_value()) + 1;
        }

        // Update our state
        self.last_sequence_read = full_number;
        if self.next_reply_expected < full_number {
            // This is most likely an event/error that allows us to update our sequence number
            // implicitly. Normally, only requests with a reply update this (in send_request()).
            self.next_reply_expected = full_number;
        }
        Some(full_number)
    }

    /// An X11 packet was received from the connection and is now enqueued into our state.
    pub(crate) fn enqueue_packet(&mut self, packet: Vec<u8>) {
        let kind = packet[0];

        // extract_sequence_number() updates our state and is thus important to call even when we
        // do not need the sequence number
        let seqno = self
            .extract_sequence_number(&packet)
            .unwrap_or(self.last_sequence_read);

        // Remove all entries for older requests
        while let Some(request) = self.sent_requests.front() {
            if request.seqno >= seqno {
                break;
            }
            let _ = self.sent_requests.pop_front();
        }
        let request = self.sent_requests.front().filter(|r| r.seqno == seqno);

        if kind == 0 {
            // It is an error. Let's see where we have to send it to.
            if let Some(request) = request {
                match request.discard_mode {
                    Some(DiscardMode::DiscardReplyAndError) => { /* This error should be ignored */
                    }
                    Some(DiscardMode::DiscardReply) => {
                        self.pending_events.push_back((seqno, packet))
                    }
                    None => self.pending_replies.push_back((seqno, packet)),
                }
            } else {
                // Unexpected error, send to main loop
                self.pending_events.push_back((seqno, packet));
            }
        } else if kind == 1 {
            // It is a reply
            if request.filter(|r| r.discard_mode.is_some()).is_some() {
                // This reply should be discarded
            } else {
                self.pending_replies.push_back((seqno, packet));
            }
        } else {
            // It is an event
            self.pending_events.push_back((seqno, packet));
        }
    }

    /// Check if the server already sent an answer to the request with the given sequence number.
    ///
    /// This function is meant to be used for requests that have a reply. Such requests always
    /// cause a reply or an error to be sent.
    pub(crate) fn poll_for_reply_or_error(&mut self, sequence: SequenceNumber) -> Option<Vec<u8>> {
        for (index, (seqno, _packet)) in self.pending_replies.iter().enumerate() {
            if *seqno == sequence {
                return Some(self.pending_replies.remove(index).unwrap().1);
            }
        }
        None
    }

    /// Prepare for calling `poll_check_for_reply_or_error()`.
    ///
    /// To check if a request with a reply caused an error, one simply has to wait for the error or
    /// reply to be received. However, this approach does not work for requests without errors:
    /// Success is indicated by the absence of an error.
    ///
    /// Thus, this function returns true if a sync is necessary to ensure that a reply with a
    /// higher sequence number will be received. Since the X11 server handles requests in-order,
    /// if the reply to a later request is received, this means that the earlier request did not
    /// fail.
    pub(crate) fn prepare_check_for_reply_or_error(
        &mut self,
        sequence: SequenceNumber,
    ) -> bool {
        if self.next_reply_expected < sequence {
            true
        } else {
            assert!(self.next_reply_expected >= sequence);
            false
        }
    }

    /// Check if the request with the given sequence number was already handled by the server.
    ///
    /// Before calling this function, you must call `prepare_check_for_reply_or_error()` with the
    /// sequence number.
    ///
    /// This function can be used for requests with and without a reply.
    pub(crate) fn poll_check_for_reply_or_error(&mut self, sequence: SequenceNumber) -> PollReply {
        if let Some(result) = self.poll_for_reply_or_error(sequence) {
            return PollReply::Reply(result);
        }

        if self.last_sequence_read > sequence {
            // We can be sure that there will be no reply/error
            PollReply::NoReply
        } else {
            // Hm, we cannot be sure yet. Perhaps there will still be a reply/error
            PollReply::TryAgain
        }
    }

    /// Find the reply for the request with the given sequence number.
    ///
    /// If the request caused an error, that error will be handled as an event. This means that a
    /// latter call to `poll_for_event()` will return it.
    pub(crate) fn poll_for_reply(&mut self, sequence: SequenceNumber) -> PollReply {
        if let Some(reply) = self.poll_for_reply_or_error(sequence) {
            if reply[0] == 0 {
                self.pending_events.push_back((sequence, reply));
                PollReply::NoReply
            } else {
                PollReply::Reply(reply)
            }
        } else {
            PollReply::TryAgain
        }
    }

    /// Get a pending event.
    pub(crate) fn poll_for_event_with_sequence(&mut self) -> Option<RawEventAndSeqNumber> {
        self.pending_events
            .pop_front()
            .map(|(seqno, event)| (GenericEvent::new(event).unwrap(), seqno))
    }
}

#[cfg(test)]
mod test {
    use super::ConnectionInner;
    use crate::connection::RequestKind;

    #[test]
    fn insert_sync_no_reply() {
        // The connection must send a sync (GetInputFocus) request every 2^16 requests (that do not
        // have a reply). Thus, this test sends more than that and tests for the sync to appear.

        // Set up a connection that writes to this array
        let mut written = [0; 0x10000 * 4 + 4];
        let mut output = &mut written[..];
        let mut connection = ConnectionInner::new(&mut output);

        for num in 1..0x10000 {
            let seqno = connection.send_request(RequestKind::IsVoid);
            assert_eq!(Some(num), seqno);
        }
        // request 0x10000 should be a sync, hence the next one is 0x10001
        let seqno = connection.send_request(RequestKind::IsVoid);
        assert_eq!(None, seqno);

        let seqno = connection.send_request(RequestKind::HasResponse);
        assert_eq!(Some(0x10000), seqno);

        let seqno = connection.send_request(RequestKind::IsVoid);
        assert_eq!(Some(0x10001), seqno);
    }

    #[test]
    fn insert_no_sync_with_reply() {
        // Compared to the previous test, this uses RequestKind::HasResponse, so no sync needs to
        // be inserted.

        // Set up a connection that writes to this array
        let mut written = [0; 0x10001 * 4];
        let mut output = &mut written[..];
        let mut connection = ConnectionInner::new(&mut output);

        for num in 1..=0x10001 {
            let seqno = connection.send_request(RequestKind::HasResponse);
            assert_eq!(Some(num), seqno);
        }
    }

    #[test]
    fn insert_no_sync_when_already_syncing() {
        // This test sends enough RequestKind::IsVoid requests that a sync becomes necessary on
        // the next request. Then it sends a RequestKind::HasResponse request so that no sync is
        // necessary. This is a regression test: Once upon a time, an unnecessary sync was done.

        // Set up a connection that writes to this array
        let mut written = [0; 0x10000 * 4];
        let mut output = &mut written[..];
        let mut connection = ConnectionInner::new(&mut output);

        for num in 1..0x10000 {
            let seqno = connection.send_request(RequestKind::IsVoid);
            assert_eq!(Some(num), seqno);
        }

        let seqno = connection.send_request(RequestKind::HasResponse);
        assert_eq!(Some(0x10000), seqno);
    }
}
