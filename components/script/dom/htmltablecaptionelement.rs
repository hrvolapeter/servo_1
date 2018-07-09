/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTableCaptionElementBinding;
use dom::bindings::root::DomRoot;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLTableCaptionElement<TH: TypeHolderTrait + 'static> {
    htmlelement: HTMLElement<TH>
}

impl<TH: TypeHolderTrait> HTMLTableCaptionElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>) -> HTMLTableCaptionElement<TH> {
        HTMLTableCaptionElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLTableCaptionElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLTableCaptionElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLTableCaptionElementBinding::Wrap)
    }
}
