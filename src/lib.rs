// Copyright 2014 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![crate_name="html5ever"]
#![crate_type="dylib"]

#![feature(plugin, box_syntax, no_std, core, collections, alloc, str_char)]
#![deny(warnings)]
#![allow(unused_parens)]

#![plugin(phf_macros)]
#![plugin(string_cache_plugin)]
#![plugin(html5ever_macros)]

// FIXME(#63): switch back to using std
#![no_std]

extern crate alloc;

#[macro_use]
extern crate core;

#[macro_use]
extern crate std;

#[cfg(for_c)]
extern crate libc;

#[macro_use]
extern crate collections;

#[cfg(not(for_c))]
#[macro_use]
extern crate log;

#[macro_use]
extern crate string_cache;

#[macro_use]
extern crate mac;

extern crate phf;

extern crate time;

extern crate iobuf;

pub use util::tendril::{Tendril, TendrilReader, TendrilReaderError, IntoTendril};
pub use tokenizer::Attribute;
pub use driver::{one_input, ParseOpts, parse_to, parse_fragment_to, parse, parse_fragment};

#[cfg(not(for_c))]
pub use serialize::serialize;

#[macro_use]
mod macros;

#[macro_use]
mod util {
    pub mod str;
    pub mod tendril;
    #[macro_use] pub mod smallcharset;
}

pub mod tokenizer;
pub mod tree_builder;

#[cfg(not(for_c))]
pub mod serialize;

/// Consumers of the parser API.
#[cfg(not(for_c))]
pub mod sink {
    pub mod common;
    pub mod rcdom;
    pub mod owned_dom;
}

pub mod driver;

#[cfg(for_c)]
pub mod for_c {
    pub mod common;
    pub mod tokenizer;
}
