// Copyright 2014 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::prelude::*;

use util::str::AsciiCast;
use util::smallcharset::SmallCharSet;

use core::str::CharRange;
use collections::string::String;
use collections::RingBuf;

pub use self::SetResult::{FromSet, NotFromSet};

struct Buffer {
    /// Byte position within the buffer.
    pub pos: uint,
    /// The buffer.
    pub buf: String,
}

/// Result from `pop_except_from`.
#[derive(PartialEq, Eq, Debug)]
pub enum SetResult {
    FromSet(char),
    NotFromSet(String),
}

/// A queue of owned string buffers, which supports incrementally
/// consuming characters.
pub struct BufferQueue {
    /// Buffers to process.
    buffers: RingBuf<Buffer>,
}

impl BufferQueue {
    /// Create an empty BufferQueue.
    pub fn new() -> BufferQueue {
        BufferQueue {
            buffers: RingBuf::with_capacity(3),
        }
    }

    /// Add a buffer to the beginning of the queue.
    pub fn push_front(&mut self, buf: String) {
        if buf.len() == 0 {
            return;
        }
        self.buffers.push_front(Buffer {
            pos: 0,
            buf: buf,
        });
    }

    /// Add a buffer to the end of the queue.
    /// 'pos' can be non-zero to remove that many bytes
    /// from the beginning.
    pub fn push_back(&mut self, buf: String, pos: uint) {
        if pos >= buf.len() {
            return;
        }
        self.buffers.push_back(Buffer {
            pos: pos,
            buf: buf,
        });
    }

    /// Look at the next available character, if any.
    pub fn peek(&mut self) -> Option<char> {
        match self.buffers.front() {
            Some(&Buffer { pos, ref buf }) => Some(buf.as_slice().char_at(pos)),
            None => None,
        }
    }

    /// Get the next character, if one is available.
    pub fn next(&mut self) -> Option<char> {
        let (result, now_empty) = match self.buffers.front_mut() {
            None => (None, false),
            Some(&mut Buffer { ref mut pos, ref buf }) => {
                let CharRange { ch, next } = buf.as_slice().char_range_at(*pos);
                *pos = next;
                (Some(ch), next >= buf.len())
            }
        };

        if now_empty {
            self.buffers.pop_front();
        }

        result
    }

    /// Pops and returns either a single character from the given set, or
    /// a `String` of characters none of which are in the set.  The set
    /// is represented as a bitmask and so can only contain the first 64
    /// ASCII characters.
    pub fn pop_except_from(&mut self, set: SmallCharSet) -> Option<SetResult> {
        let (result, now_empty) = match self.buffers.front_mut() {
            Some(&mut Buffer { ref mut pos, ref buf }) => {
                let n = set.nonmember_prefix_len(&buf.as_slice()[*pos..]);
                if n > 0 {
                    let new_pos = *pos + n;
                    let out = String::from_str(&buf.as_slice()[*pos..new_pos]);
                    *pos = new_pos;
                    (Some(NotFromSet(out)), new_pos >= buf.len())
                } else {
                    let CharRange { ch, next } = buf.as_slice().char_range_at(*pos);
                    *pos = next;
                    (Some(FromSet(ch)), next >= buf.len())
                }
            }
            _ => (None, false),
        };

        // Unborrow self for this part.
        if now_empty {
            self.buffers.pop_front();
        }

        result
    }

    // Check if the next characters are an ASCII case-insensitive match for
    // `pat`, which must be non-empty.
    //
    // If so, consume them and return Some(true).
    // If they do not match, return Some(false).
    // If not enough characters are available to know, return None.
    pub fn eat(&mut self, pat: &str) -> Option<bool> {
        let mut buffers_exhausted = 0u;
        let mut consumed_from_last = match self.buffers.front() {
            None => return None,
            Some(ref buf) => buf.pos,
        };

        for c in pat.chars() {
            if buffers_exhausted >= self.buffers.len() {
                return None;
            }
            let ref buf = self.buffers[buffers_exhausted];

            let d = buf.buf.as_slice().char_at(consumed_from_last);
            match (c.to_ascii_opt(), d.to_ascii_opt()) {
                (Some(c), Some(d)) => if c.eq_ignore_case(d) { () } else { return Some(false) },
                _ => return Some(false),
            }

            // d was an ASCII character; size must be 1 byte
            consumed_from_last += 1;
            if consumed_from_last >= buf.buf.len() {
                buffers_exhausted += 1;
                consumed_from_last = 0;
            }
        }

        // We have a match. Commit changes to the BufferQueue.
        for _ in range(0, buffers_exhausted) {
            self.buffers.pop_front();
        }

        match self.buffers.front_mut() {
            None => assert_eq!(consumed_from_last, 0),
            Some(ref mut buf) => buf.pos = consumed_from_last,
        }

        Some(true)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use core::prelude::*;
    use collections::string::String;
    use super::{BufferQueue, FromSet, NotFromSet};

    #[test]
    fn smoke_test() {
        let mut bq = BufferQueue::new();
        assert_eq!(bq.peek(), None);
        assert_eq!(bq.next(), None);

        bq.push_back(String::from_str("abc"), 0);
        assert_eq!(bq.peek(), Some('a'));
        assert_eq!(bq.next(), Some('a'));
        assert_eq!(bq.peek(), Some('b'));
        assert_eq!(bq.peek(), Some('b'));
        assert_eq!(bq.next(), Some('b'));
        assert_eq!(bq.peek(), Some('c'));
        assert_eq!(bq.next(), Some('c'));
        assert_eq!(bq.peek(), None);
        assert_eq!(bq.next(), None);
    }

    #[test]
    fn can_unconsume() {
        let mut bq = BufferQueue::new();
        bq.push_back(String::from_str("abc"), 0);
        assert_eq!(bq.next(), Some('a'));

        bq.push_front(String::from_str("xy"));
        assert_eq!(bq.next(), Some('x'));
        assert_eq!(bq.next(), Some('y'));
        assert_eq!(bq.next(), Some('b'));
        assert_eq!(bq.next(), Some('c'));
        assert_eq!(bq.next(), None);
    }

    #[test]
    fn can_pop_except_set() {
        let mut bq = BufferQueue::new();
        bq.push_back(String::from_str("abc&def"), 0);
        let mut pop = |&mut:| bq.pop_except_from(small_char_set!('&'));
        assert_eq!(pop(), Some(NotFromSet(String::from_str("abc"))));
        assert_eq!(pop(), Some(FromSet('&')));
        assert_eq!(pop(), Some(NotFromSet(String::from_str("def"))));
        assert_eq!(pop(), None);
    }

    #[test]
    fn can_push_truncated() {
        let mut bq = BufferQueue::new();
        bq.push_back(String::from_str("abc"), 1);
        assert_eq!(bq.next(), Some('b'));
        assert_eq!(bq.next(), Some('c'));
        assert_eq!(bq.next(), None);
    }

    #[test]
    fn can_eat() {
        // This is not very comprehensive.  We rely on the tokenizer
        // integration tests for more thorough testing with many
        // different input buffer splits.
        let mut bq = BufferQueue::new();
        bq.push_back(String::from_str("a"), 0);
        bq.push_back(String::from_str("bc"), 0);
        assert_eq!(bq.eat("abcd"), None);
        assert_eq!(bq.eat("ax"), Some(false));
        assert_eq!(bq.eat("ab"), Some(true));
        assert_eq!(bq.next(), Some('c'));
        assert_eq!(bq.next(), None);
    }
}
