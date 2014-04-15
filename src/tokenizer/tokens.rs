/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::strbuf::StrBuf;
use util::str::empty_str;

// FIXME: already exists in Servo DOM
#[deriving(Eq, TotalEq, Clone)]
pub struct Doctype {
    name: Option<StrBuf>,
    public_id: Option<StrBuf>,
    system_id: Option<StrBuf>,
    force_quirks: bool,
}

impl Doctype {
    pub fn new() -> Doctype {
        Doctype {
            name: None,
            public_id: None,
            system_id: None,
            force_quirks: false,
        }
    }
}

#[deriving(Eq, TotalEq, Clone)]
pub struct Attribute {
    name: StrBuf,
    value: StrBuf,
}

impl Attribute {
    pub fn new() -> Attribute {
        Attribute {
            name: empty_str(),
            value: empty_str(),
        }
    }

    pub fn clear(&mut self) {
        self.name.truncate(0);
        self.value.truncate(0);
    }
}

#[deriving(Eq, TotalEq, Clone)]
pub enum TagKind {
    StartTag,
    EndTag,
}

#[deriving(Eq, TotalEq, Clone)]
pub struct Tag {
    kind: TagKind,
    name: StrBuf,
    self_closing: bool,
    attrs: Vec<Attribute>,
}

impl Tag {
    pub fn new(kind: TagKind) -> Tag {
        Tag {
            kind: kind,
            name: empty_str(),
            self_closing: false,
            attrs: Vec::new(),
        }
    }
}

#[deriving(Eq, TotalEq, Clone)]
pub enum Token {
    DoctypeToken(Doctype),
    TagToken(Tag),
    CommentToken(StrBuf),
    CharacterToken(char),
    MultiCharacterToken(StrBuf),
    EOFToken,
    ParseError(~str),
}
