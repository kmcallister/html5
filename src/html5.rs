/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_id="github.com/kmcallister/html5"]
#![crate_type="dylib"]

#![feature(macro_rules, phase)]

#[phase(syntax, link)]
extern crate log;

#[phase(syntax)]
extern crate phf_mac;

#[phase(syntax)]
extern crate macros = "html5-macros";

extern crate phf;
extern crate collections;
extern crate time;

mod util {
    pub mod str;
}

pub mod tokenizer;
