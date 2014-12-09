// Copyright 2014 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::prelude::*;

use tokenizer::states;
use util::span::Span;

use collections::vec::Vec;
use collections::slice::OrdSliceAllocPrelude;
use collections::str::MaybeOwned;

use string_cache::{Atom, QualName};

pub use self::TagKind::*;
pub use self::Token::*;

/// A `DOCTYPE` token.
// FIXME: already exists in Servo DOM
#[deriving(PartialEq, Eq, Clone, Show)]
pub struct Doctype {
    pub name:      Option<Atom>,
    pub public_id: Option<Span>,
    pub system_id: Option<Span>,
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
#[deriving(PartialEq, Eq, PartialOrd, Ord, Clone, Show)]
pub struct Attribute {
    pub name: QualName,
    pub value: Span,
}

#[deriving(PartialEq, Eq, Clone, Show)]
pub enum TagKind {
    StartTag,
    EndTag,
}

/// A tag token.
#[deriving(PartialEq, Eq, Clone, Show)]
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
        self_attrs.as_mut_slice().sort();
        other_attrs.as_mut_slice().sort();

        self_attrs == other_attrs
    }
}

#[deriving(PartialEq, Eq, Clone, Show)]
pub enum Token {
    DoctypeToken(Doctype),
    TagToken(Tag),
    CommentToken(Span),
    CharacterTokens(Span),
    NullCharacterToken,
    EOFToken,
    ParseError(MaybeOwned<'static>),
}

/// Types which can receive tokens from the tokenizer.
pub trait TokenSink {
    /// Process a token.
    fn process_token(&mut self, token: Token);

    /// The tokenizer will call this after emitting any start tag.
    /// This allows the tree builder to change the tokenizer's state.
    /// By default no state changes occur.
    fn query_state_change(&mut self) -> Option<states::State> {
        None
    }
}
