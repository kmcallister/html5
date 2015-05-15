// Copyright 2014 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use tokenizer::states;

use std::borrow::Cow;

use string_cache::{Atom, QualName};

pub use self::TagKind::{StartTag, EndTag};
pub use self::Token::{DoctypeToken, TagToken, CommentToken, CharacterTokens};
pub use self::Token::{NullCharacterToken, EOFToken, ParseError};

pub use self::XTagKind::{StartXTag, EndXTag, EmptyXTag, ShortXTag};
pub use self::XToken::{DoctypeXToken, XTagToken, PIToken, CommentXToken};
pub use self::XToken::{CharacterXTokens, EOFXToken, XParseError, NullCharacterXToken};


/// A `DOCTYPE` token.
// FIXME: already exists in Servo DOM
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Doctype {
    pub name: Option<String>,
    pub public_id: Option<String>,
    pub system_id: Option<String>,
    pub force_quirks: bool,
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

/// A tag attribute.
///
/// The namespace on the attribute name is almost always ns!("").
/// The tokenizer creates all attributes this way, but the tree
/// builder will adjust certain attribute names inside foreign
/// content (MathML, SVG).
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct Attribute {
    pub name: QualName,
    pub value: String,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum TagKind {
    StartTag,
    EndTag,
}

/// A tag token.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Tag {
    pub kind: TagKind,
    pub name: Atom,
    pub self_closing: bool,
    pub attrs: Vec<Attribute>,
}

impl Tag {
    /// Are the tags equivalent when we don't care about attribute order?
    /// Also ignores the self-closing flag.
    pub fn equiv_modulo_attr_order(&self, other: &Tag) -> bool {
        if (self.kind != other.kind) || (self.name != other.name) {
            return false;
        }

        let mut self_attrs = self.attrs.clone();
        let mut other_attrs = other.attrs.clone();
        self_attrs.sort();
        other_attrs.sort();

        self_attrs == other_attrs
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Token {
    DoctypeToken(Doctype),
    TagToken(Tag),
    CommentToken(String),
    CharacterTokens(String),
    NullCharacterToken,
    EOFToken,
    ParseError(Cow<'static, str>),
}

// FIXME: rust-lang/rust#22629
unsafe impl Send for Token { }

/// Types which can receive tokens from the tokenizer.
pub trait TokenSink {
    /// Process a token.
    fn process_token(&mut self, token: Token);

    /// The tokenizer will call this after emitting any tag.
    /// This allows the tree builder to change the tokenizer's state.
    /// By default no state changes occur.
    fn query_state_change(&mut self) -> Option<states::State> {
        None
    }
}


#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum XTagKind {
    StartXTag,
    EndXTag,
    EmptyXTag,
    ShortXTag,
}

/// XML 5 Tag Token
// FIXME: Possibly unify with Tag?
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct XTag {
    pub kind: XTagKind,
    pub name: Atom,
    pub attrs: Vec<Attribute>
}

impl XTag {
    pub fn equiv_modulo_attr_order(&self, other: &XTag) -> bool {
        if (self.kind != other.kind) || (self.name != other.name) {
            return false;
        }

        let mut self_attrs = self.attrs.clone();
        let mut other_attrs = other.attrs.clone();
        self_attrs.sort();
        other_attrs.sort();

        self_attrs == other_attrs
    }
}

// FIXME: rust-lang/rust#22629
unsafe impl Send for XToken { }

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct XPi {
    pub target: String,
    pub data: String,
}

#[derive(PartialEq, Eq, Debug)]
pub enum XToken {
    DoctypeXToken(Doctype),
    XTagToken(XTag),
    PIToken(XPi),
    CommentXToken(String),
    CharacterXTokens(String),
    EOFXToken,
    NullCharacterXToken,
    XParseError(Cow<'static, str>),
}

/// Types which can receive tokens from the tokenizer.
pub trait XTokenSink {
    /// Process a token.
    fn process_token(&mut self, token: XToken);

    /// The tokenizer will call this after emitting any start tag.
    /// This allows the tree builder to change the tokenizer's state.
    /// By default no state changes occur.
    fn query_state_change(&mut self) -> Option<states::XmlState> {
        None
    }
}
