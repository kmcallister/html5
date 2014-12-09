// Copyright 2014 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use util::foreach_html5lib_test;

use std::{num, char};
use std::mem::replace;
use std::default::Default;
use std::path::Path;
use test::{TestDesc, TestDescAndFn, DynTestName, DynTestFn};
use serialize::json;
use serialize::json::Json;
use std::collections::TreeMap;
use std::str::Slice;
use std::vec::MoveItems;

use html5ever::{Span, BufSpan, ROIobuf};
use html5ever::tokenizer::{Doctype, Attribute, StartTag, EndTag, Tag};
use html5ever::tokenizer::{Token, DoctypeToken, TagToken, CommentToken};
use html5ever::tokenizer::{CharacterTokens, NullCharacterToken, EOFToken, ParseError};
use html5ever::tokenizer::{TokenSink, Tokenizer, TokenizerOpts};
use html5ever::tokenizer::states::{Plaintext, RawData, Rcdata, Rawtext};

use string_cache::{Atom, QualName};

// Return all ways of splitting the string into at most n
// possibly-empty pieces.
fn splits(s: &str, n: uint) -> Vec<Vec<String>> {
    if n == 1 {
        return vec!(vec!(s.to_string()));
    }

    let mut points: Vec<uint> = s.char_indices().map(|(n,_)| n).collect();
    points.push(s.len());

    // do this with iterators?
    let mut out = vec!();
    for p in points.into_iter() {
        let y = s.slice_from(p);
        for mut x in splits(s.slice_to(p), n-1).into_iter() {
            x.push(y.to_string());
            out.push(x);
        }
    }

    out.extend(splits(s, n-1).into_iter());
    out
}

struct TokenLogger {
    tokens: Vec<Token>,
    current_str: Span,
    exact_errors: bool,
}

impl TokenLogger {
    fn new(exact_errors: bool) -> TokenLogger {
        TokenLogger {
            tokens: vec!(),
            current_str: BufSpan::new(),
            exact_errors: exact_errors,
        }
    }

    // Push anything other than character tokens
    fn push(&mut self, token: Token) {
        self.finish_str();
        self.tokens.push(token);
    }

    fn finish_str(&mut self) {
        if !self.current_str.is_empty() {
            let s = replace(&mut self.current_str, BufSpan::new());
            self.tokens.push(CharacterTokens(s));
        }
    }

    fn get_tokens(mut self) -> Vec<Token> {
        self.finish_str();
        self.tokens
    }
}

impl TokenSink for TokenLogger {
    fn process_token(&mut self, token: Token) {
        match token {
            CharacterTokens(b) => {
                self.current_str.append(b);
            }

            NullCharacterToken => {
                self.current_str.push(ROIobuf::from_str("\0"));
            }

            ParseError(_) => if self.exact_errors {
                self.push(ParseError(Slice("")));
            },

            TagToken(mut t) => {
                // The spec seems to indicate that one can emit
                // erroneous end tags with attrs, but the test
                // cases don't contain them.
                match t.kind {
                    EndTag => {
                        t.self_closing = false;
                        t.attrs = vec!();
                    }
                    _ => t.attrs.sort_by(|a1, a2| a1.name.cmp(&a2.name)),
                }
                self.push(TagToken(t));
            }

            EOFToken => (),

            _ => self.push(token),
        }
    }
}

fn tokenize(input: Vec<ROIobuf<'static>>, opts: TokenizerOpts) -> Vec<Token> {
    let sink = TokenLogger::new(opts.exact_errors);
    let mut tok = Tokenizer::new(sink, opts);
    for chunk in input.into_iter() {
        tok.feed(chunk);
    }
    tok.end();
    tok.unwrap().get_tokens()
}

trait JsonExt {
    fn get_str(&self) -> &str;
    fn get_span(&self) -> Span;
    fn get_nullable_str(&self) -> Option<&str>;
    fn get_nullable_span(&self) -> Option<Span>;
    fn get_nullable_atom(&self) -> Option<Atom>;
    fn get_bool(&self) -> bool;
    fn get_obj<'t>(&'t self) -> &'t TreeMap<String, Self>;
    fn get_list<'t>(&'t self) -> &'t Vec<Self>;
    fn find<'t>(&'t self, key: &str) -> &'t Self;
}

impl JsonExt for Json {
    fn get_str(&self) -> &str {
        match *self {
            json::String(ref s) => s.as_slice(),
            _ => panic!("Json::get_str: not a String"),
        }
    }

    fn get_span(&self) -> Span {
        BufSpan::from_buf(ROIobuf::from_str_copy(self.get_str()))
    }

    fn get_nullable_str(&self) -> Option<&str> {
        match *self {
            json::Null => None,
            json::String(ref s) => Some(s.as_slice()),
            _ => panic!("Json::get_nullable_str: not a String"),
        }
    }

    fn get_nullable_span(&self) -> Option<Span> {
        self.get_nullable_str().map(|s| BufSpan::from_buf(ROIobuf::from_str_copy(s)))
    }

    fn get_nullable_atom(&self) -> Option<Atom> {
        self.get_nullable_str().map(Atom::from_slice)
    }

    fn get_bool(&self) -> bool {
        match *self {
            json::Boolean(b) => b,
            _ => panic!("Json::get_bool: not a Boolean"),
        }
    }

    fn get_obj<'t>(&'t self) -> &'t TreeMap<String, Json> {
        match *self {
            json::Object(ref m) => &*m,
            _ => panic!("Json::get_obj: not an Object"),
        }
    }

    fn get_list<'t>(&'t self) -> &'t Vec<Json> {
        match *self {
            json::Array(ref m) => m,
            _ => panic!("Json::get_list: not an Array"),
        }
    }

    fn find<'t>(&'t self, key: &str) -> &'t Json {
        self.get_obj().get(&key.to_string()).unwrap()
    }
}

// Parse a JSON object (other than "ParseError") to a token.
fn json_to_token(js: &Json) -> Token {
    let parts = js.get_list();
    // Collect refs here so we don't have to use "ref" in all the patterns below.
    let args: Vec<&Json> = parts.slice_from(1).iter().collect();
    match (parts[0].get_str().as_slice(), args.as_slice()) {
        ("DOCTYPE", [name, public_id, system_id, correct]) => DoctypeToken(Doctype {
            name: name.get_nullable_atom(),
            public_id: public_id.get_nullable_span(),
            system_id: system_id.get_nullable_span(),
            force_quirks: !correct.get_bool(),
        }),

        ("StartTag", [name, attrs, rest..]) => TagToken(Tag {
            kind: StartTag,
            name: Atom::from_slice(name.get_str().as_slice()),
            attrs: attrs.get_obj().iter().map(|(k,v)| {
                Attribute {
                    name: QualName::new(ns!(""), Atom::from_slice(k.as_slice())),
                    value: v.get_span()
                }
            }).collect(),
            self_closing: match rest {
                [ref b, ..] => b.get_bool(),
                _ => false,
            }
        }),

        ("EndTag", [name]) => TagToken(Tag {
            kind: EndTag,
            name: Atom::from_slice(name.get_str().as_slice()),
            attrs: vec!(),
            self_closing: false
        }),

        ("Comment", [txt]) => CommentToken(txt.get_span()),

        ("Character", [txt]) => CharacterTokens(txt.get_span()),

        // We don't need to produce NullCharacterToken because
        // the TokenLogger will convert them to CharacterTokens.

        _ => panic!("don't understand token {}", parts),
    }
}

// Parse the "output" field of the test case into a vector of tokens.
fn json_to_tokens(js: &Json, exact_errors: bool) -> Vec<Token> {
    // Use a TokenLogger so that we combine character tokens separated
    // by an ignored error.
    let mut sink = TokenLogger::new(exact_errors);
    for tok in js.get_list().iter() {
        match *tok {
            json::String(ref s)
                if s.as_slice() == "ParseError" => sink.process_token(ParseError(Slice(""))),
            _ => sink.process_token(json_to_token(tok)),
        }
    }
    sink.get_tokens()
}

// Undo the escaping in "doubleEscaped" tests.
fn unescape(s: &str) -> Option<String> {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars().peekable();
    loop {
        match it.next() {
            None => return Some(out),
            Some('\\') => {
                if it.peek() != Some(&'u') {
                    panic!("can't understand escape");
                }
                drop(it.next());
                let hex: String = it.by_ref().take(4).collect();
                match num::from_str_radix(hex.as_slice(), 16)
                          .and_then(char::from_u32) {
                    // Some of the tests use lone surrogates, but we have no
                    // way to represent them in the UTF-8 input to our parser.
                    // Since these can only come from script, we will catch
                    // them there.
                    None => return None,
                    Some(c) => out.push(c),
                }
            }
            Some(c) => out.push(c),
        }
    }
}

fn unescape_json(js: &Json) -> Json {
    match *js {
        // unwrap is OK here because the spec'd *output* of the tokenizer never
        // contains a lone surrogate.
        json::String(ref s) => json::String(unescape(s.as_slice()).unwrap()),
        json::Array(ref xs) => json::Array(xs.iter().map(unescape_json).collect()),
        json::Object(ref obj) => {
            let mut new_obj = TreeMap::new();
            for (k,v) in obj.iter() {
                new_obj.insert(k.clone(), unescape_json(v));
            }
            json::Object(new_obj)
        }
        _ => js.clone(),
    }
}
fn mk_test(desc: String, insplits: Vec<Vec<String>>, expect: Json, exact_errors: bool, opts: TokenizerOpts)
        -> TestDescAndFn {
    TestDescAndFn {
        desc: TestDesc {
            name: DynTestName(desc),
            ignore: false,
            should_fail: false,
        },
        testfn: DynTestFn(proc() {
            let expect_toks = json_to_tokens(&expect, exact_errors);

            for input in insplits.into_iter() {
                // Clone 'input' so we have it for the failure message.
                // Also clone opts.  If we don't, we get the wrong
                // result but the compiler doesn't catch it!
                // Possibly mozilla/rust#12223.
                let input_bufs = input.clone().into_iter().map(|x| ROIobuf::from_str_copy(x.as_slice())).collect();
                let output = tokenize(input_bufs, opts.clone());
                if output != expect_toks {
                    panic!("\ninput: {}\ngot: {}\nexpected: {}",
                        input, output, expect_toks);
                }
            }
        }),
    }
}

fn mk_tests(tests: &mut Vec<TestDescAndFn>, path_str: &str, js: &Json) {
    let obj = js.get_obj();
    let mut input = js.find("input").unwrap().get_str().to_string();

    let mut expect = js.find("output").unwrap().clone();
    let desc = format!("tok: {}: {}",
        path_str, js.find("description").unwrap().get_str());

    // "Double-escaped" tests require additional processing of
    // the input and output.
    if obj.get(&"doubleEscaped".to_string()).map_or(false, |j| j.get_bool()) {
        match unescape(input.as_slice()) {
            None => return,
            Some(i) => input = i,
        }
        expect = unescape_json(&expect);
    }

    // Split up the input at different points to test incremental tokenization.
    let insplits = splits(input.as_slice(), 3);

    // Some tests have a last start tag name.
    let start_tag = obj.get(&"lastStartTag".to_string()).map(|s| s.get_str());

    // Some tests want to start in a state other than Data.
    let state_overrides = match obj.get(&"initialStates".to_string()) {
        Some(&json::Array(ref xs)) => xs.iter().map(|s|
            Some(match s.get_str().as_slice() {
                "PLAINTEXT state" => Plaintext,
                "RAWTEXT state"   => RawData(Rawtext),
                "RCDATA state"    => RawData(Rcdata),
                s => panic!("don't know state {}", s),
            })).collect(),
        None => vec!(None),
        _ => panic!("don't understand initialStates value"),
    };

    // Build the tests.
    for state in state_overrides.into_iter() {
        for &exact_errors in [false, true].iter() {
            let mut newdesc = desc.clone();
            match state {
                Some(s) => newdesc = format!("{} (in state {})", newdesc, s),
                None  => (),
            };
            if exact_errors {
                newdesc = format!("{} (exact errors)", newdesc);
            }

            tests.push(mk_test(newdesc, insplits.clone(), expect.clone(), exact_errors, TokenizerOpts {
                exact_errors: exact_errors,
                initial_state: state,
                last_start_tag_name: start_tag.map(|s| s.to_string()),

                // Not discarding a BOM is what the test suite expects; see
                // https://github.com/html5lib/html5lib-tests/issues/2
                discard_bom: false,

                .. Default::default()
            }));
        }
    }
}

pub fn tests(src_dir: Path) -> MoveItems<TestDescAndFn> {
    let mut tests = vec!();

    foreach_html5lib_test(src_dir, "tokenizer", ".test", |path_str, mut file| {
        let js = json::from_reader(&mut file as &mut Reader)
            .ok().expect("json parse error");

        match js.get_obj().get(&"tests".to_string()) {
            Some(&json::Array(ref lst)) => {
                for test in lst.iter() {
                    mk_tests(&mut tests, path_str.as_slice(), test);
                }
            }

            // xmlViolation.test doesn't follow this format.
            _ => (),
        }
    });

    tests.into_iter()
}
