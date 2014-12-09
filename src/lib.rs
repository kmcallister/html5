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

#![feature(macro_rules, phase, globs, unsafe_destructor)]
#![deny(warnings)]
#![allow(unused_parens)]

// Don't implicitly pull in things from std::*
// This helps us make a C-friendly library.
#![no_std]

extern crate alloc;

#[phase(plugin, link)]
extern crate core;

#[cfg(not(for_c))]
#[phase(plugin, link)]
extern crate std;

#[cfg(for_c)]
extern crate libc;

#[phase(plugin, link)]
extern crate collections;

#[cfg(not(for_c))]
#[phase(plugin, link)]
extern crate log;

extern crate iobuf;

#[phase(plugin)]
extern crate phf_mac;

#[phase(plugin)]
extern crate string_cache_macros;
extern crate string_cache;

#[phase(plugin)]
extern crate html5ever_macros;

extern crate phf;

extern crate time;

pub use tokenizer::Attribute;
pub use driver::{one_input, ParseOpts, parse_to, parse};

#[cfg(not(for_c))]
pub use serialize::serialize;

pub use iobuf::{BufSpan, Iobuf, ROIobuf};
pub use util::span::{Buf, Span, ValidatedSpanUtils};

mod macros;

mod util {
    #![macro_escape]

    pub mod fast_option;
    pub mod single_char;
    pub mod span;
    pub mod str;
    pub mod smallcharset;
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

/// A fake `std` module so that `deriving` and other macros will work.
/// See rust-lang/rust#16803.
#[cfg(for_c)]
mod std {
    pub use core::{clone, cmp, default, fmt, option, str};
    pub use collections::hash;
}
