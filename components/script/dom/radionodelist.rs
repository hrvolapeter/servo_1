/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use dom::bindings::codegen::Bindings::RadioNodeListBinding;
use dom::bindings::codegen::Bindings::RadioNodeListBinding::RadioNodeListMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::{Dom, DomRoot};
use dom::bindings::str::DOMString;
use dom::htmlinputelement::{HTMLInputElement, InputType};
use dom::node::Node;
use dom::nodelist::{NodeList, NodeListType};
use dom::window::Window;
use dom_struct::dom_struct;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct RadioNodeList<TH: TypeHolderTrait + 'static> {
    node_list: NodeList<TH>,
}

impl<TH: TypeHolderTrait> RadioNodeList<TH> {
    #[allow(unrooted_must_root)]
    fn new_inherited(list_type: NodeListType<TH>) -> RadioNodeList<TH> {
        RadioNodeList {
            node_list: NodeList::new_inherited(list_type)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window<TH>, list_type: NodeListType<TH>) -> DomRoot<RadioNodeList<TH>> {
        reflect_dom_object(Box::new(RadioNodeList::new_inherited(list_type)),
                           window,
                           RadioNodeListBinding::Wrap)
    }

    pub fn new_simple_list<T>(window: &Window<TH>, iter: T) -> DomRoot<RadioNodeList<TH>>
                              where T: Iterator<Item=DomRoot<Node<TH>>> {
        RadioNodeList::new(window, NodeListType::Simple(iter.map(|r| Dom::from_ref(&*r)).collect()))
    }

    // FIXME: This shouldn't need to be implemented here since NodeList (the parent of
    // RadioNodeList) implements Length
    // https://github.com/servo/servo/issues/5875
    pub fn Length(&self) -> u32 {
        self.node_list.Length()
    }
}

impl<TH: TypeHolderTrait> RadioNodeListMethods<TH> for RadioNodeList<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-radionodelist-value
    fn Value(&self) -> DOMString {
        self.upcast::<NodeList<TH>>().as_simple_list().iter().filter_map(|node| {
            // Step 1
            node.downcast::<HTMLInputElement<TH>>().and_then(|input| {
                if input.input_type() == InputType::Radio && input.Checked() {
                    // Step 3-4
                    let value = input.Value();
                    Some(if value.is_empty() { DOMString::from("on") } else { value })
                } else {
                    None
                }
            })
        }).next()
        // Step 2
          .unwrap_or(DOMString::from(""))
    }

    // https://html.spec.whatwg.org/multipage/#dom-radionodelist-value
    fn SetValue(&self, value: DOMString) {
        for node in self.upcast::<NodeList<TH>>().as_simple_list().iter() {
            // Step 1
            if let Some(input) = node.downcast::<HTMLInputElement<TH>>() {
                match input.input_type() {
                    InputType::Radio if value == DOMString::from("on") => {
                        // Step 2
                        let val = input.Value();
                        if val.is_empty() || val == value {
                            input.SetChecked(true);
                            return;
                        }
                    }
                    InputType::Radio => {
                        // Step 2
                        if input.Value() == value {
                            input.SetChecked(true);
                            return;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // FIXME: This shouldn't need to be implemented here since NodeList (the parent of
    // RadioNodeList) implements IndexedGetter.
    // https://github.com/servo/servo/issues/5875
    //
    // https://dom.spec.whatwg.org/#dom-nodelist-item
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<Node<TH>>> {
        self.node_list.IndexedGetter(index)
    }
}
