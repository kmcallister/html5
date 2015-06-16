// Copyright 2014 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(plugin, collections)]
#![plugin(string_cache_plugin)]

extern crate html5ever;
extern crate html5ever_dom_sink;

#[macro_use]
extern crate string_cache;
extern crate tendril;

use std::io::{self, Read};
use std::iter::repeat;
use std::default::Default;
use std::string::String;

use tendril::{ByteTendril, ReadExt};
use html5ever::{parse, one_input};
use html5ever_dom_sink::common::{Document, Doctype, Text, Comment, Element};
use html5ever_dom_sink::rcdom::{RcDom, Handle};

// This is not proper HTML serialization, of course.

fn walk(indent: usize, handle: Handle) {
    let node = handle.borrow();
    // FIXME: don't allocate
    print!("{}", repeat(" ").take(indent).collect::<String>());
    match node.node {
        Document
            => println!("#Document"),

        Doctype(ref name, ref public, ref system)
            => println!("<!DOCTYPE {} \"{}\" \"{}\">", *name, *public, *system),

        Text(ref text)
            => println!("#text: {}", text.escape_default()),

        Comment(ref text)
            => println!("<!-- {} -->", text.escape_default()),

        Element(ref name, ref attrs) => {
            assert!(name.ns == ns!(html));
            print!("<{}", name.local);
            for attr in attrs.iter() {
                assert!(attr.name.ns == ns!(""));
                print!(" {}=\"{}\"", attr.name.local, attr.value);
            }
            println!(">");
        }
    }

    for child in node.children.iter() {
        walk(indent+4, child.clone());
    }
}

fn main() {
    let mut input = ByteTendril::new();
    io::stdin().read_to_tendril(&mut input).unwrap();
    let input = input.try_reinterpret().unwrap();
    let dom: RcDom = parse(one_input(input), Default::default());
    walk(0, dom.document);

    if !dom.errors.is_empty() {
        println!("\nParse errors:");
        for err in dom.errors.into_iter() {
            println!("    {}", err);
        }
    }
}
