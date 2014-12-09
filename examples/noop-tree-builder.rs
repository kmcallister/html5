// Copyright 2014 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate string_cache;

extern crate html5ever;

use std::io;
use std::default::Default;
use std::collections::HashMap;
use std::str::MaybeOwned;
use string_cache::{QualName, Atom};

use html5ever::{ROIobuf, parse_to, one_input, Span};
use html5ever::tokenizer::Attribute;
use html5ever::tree_builder::{TreeSink, QuirksMode, NodeOrText};

struct Sink {
    next_id: uint,
    names: HashMap<uint, QualName>,
}

impl Sink {
    fn get_id(&mut self) -> uint {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

impl TreeSink<uint> for Sink {
    fn get_document(&mut self) -> uint {
        0
    }

    fn same_node(&self, x: uint, y: uint) -> bool {
        x == y
    }

    fn elem_name(&self, target: uint) -> QualName {
        self.names.get(&target).expect("not an element").clone()
    }

    fn create_element(&mut self, name: QualName, _attrs: Vec<Attribute>) -> uint {
        let id = self.get_id();
        self.names.insert(id, name);
        id
    }

    fn create_comment(&mut self, _text: Span) -> uint {
        self.get_id()
    }

    fn append_before_sibling(&mut self,
            _sibling: uint,
            _new_node: NodeOrText<uint>) -> Result<(), NodeOrText<uint>> {
        // `sibling` will have a parent unless a script moved it, and we're
        // not running scripts.  Therefore we can aways return `Ok(())`.
        Ok(())
    }

    fn parse_error(&mut self, _msg: MaybeOwned<'static>) { }
    fn set_quirks_mode(&mut self, _mode: QuirksMode) { }
    fn append(&mut self, _parent: uint, _child: NodeOrText<uint>) { }

    fn append_doctype_to_document(&mut self, _name: Atom, _public_id: Span, _system_id: Span) { }
    fn add_attrs_if_missing(&mut self, _target: uint, _attrs: Vec<Attribute>) { }
    fn remove_from_parent(&mut self, _target: uint) { }
    fn mark_script_already_started(&mut self, _node: uint) { }
}

fn main() {
    let sink = Sink {
        next_id: 1,
        names: HashMap::new(),
    };

    let input = io::stdin().read_to_string().unwrap();
    parse_to(sink, one_input(ROIobuf::from_str_copy(input.as_slice())), Default::default());
}
