/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unrooted_must_root)]

use dom::bindings::root::{Dom, DomRoot};
use dom::bindings::trace::JSTraceable;
use dom::document::Document;
use dom::htmlscriptelement::HTMLScriptElement;
use dom::node::Node;
use dom::servoparser::{ParsingAlgorithm, Sink};
use js::jsapi::JSTracer;
use servo_url::ServoUrl;
use xml5ever::buffer_queue::BufferQueue;
use xml5ever::tokenizer::XmlTokenizer;
use xml5ever::tree_builder::{Tracer as XmlTracer, XmlTreeBuilder};
use typeholder::TypeHolderTrait;

#[derive(JSTraceable, MallocSizeOf)]
#[must_root]
pub struct Tokenizer<TH: TypeHolderTrait + 'static> {
    #[ignore_malloc_size_of = "Defined in xml5ever"]
    inner: XmlTokenizer<XmlTreeBuilder<Dom<Node<TH>>, Sink<TH>>>,
}

impl<TH: TypeHolderTrait> Tokenizer<TH> {
    pub fn new(document: &Document<TH>, url: ServoUrl) -> Self {
        let sink = Sink {
            base_url: url,
            document: Dom::from_ref(document),
            current_line: 1,
            script: Default::default(),
            parsing_algorithm: ParsingAlgorithm::Normal,
        };

        let tb = XmlTreeBuilder::new(sink, Default::default());
        let tok = XmlTokenizer::new(tb, Default::default());

        Tokenizer {
            inner: tok,
        }
    }

    pub fn feed(&mut self, input: &mut BufferQueue) -> Result<(), DomRoot<HTMLScriptElement<TH>>> {
        if !input.is_empty() {
            while let Some(chunk) = input.pop_front() {
                self.inner.feed(chunk);
                if let Some(script) = self.inner.sink.sink.script.take() {
                    return Err(script);
                }
            }
        } else {
            self.inner.run();
            if let Some(script) = self.inner.sink.sink.script.take() {
                return Err(script);
            }
        }
        Ok(())
    }

    pub fn end(&mut self) {
        self.inner.end()
    }

    pub fn url(&self) -> &ServoUrl {
        &self.inner.sink.sink.base_url
    }
}

#[allow(unsafe_code)]
unsafe impl<TH: TypeHolderTrait> JSTraceable for XmlTokenizer<XmlTreeBuilder<Dom<Node<TH>>, Sink<TH>>> {
    unsafe fn trace<TH>(&self, trc: *mut JSTracer) {
        struct Tracer<TH>(*mut JSTracer);
        let tracer = Tracer(trc);

        impl<TH: TypeHolderTrait> XmlTracer for Tracer<TH> {
            type Handle = Dom<Node<TH>>;
            #[allow(unrooted_must_root)]
            fn trace_handle(&self, node: &Dom<Node<TH>>) {
                unsafe { node.trace(self.0); }
            }
        }

        let tree_builder = &self.sink;
        tree_builder.trace_handles(&tracer);
        tree_builder.sink.trace(trc);
    }
}
