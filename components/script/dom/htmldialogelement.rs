/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::HTMLDialogElementBinding;
use dom::bindings::codegen::Bindings::HTMLDialogElementBinding::HTMLDialogElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::Element;
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, window_from_node};
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLDialogElement<TH: TypeHolderTrait + 'static> {
    htmlelement: HTMLElement<TH>,
    return_value: DomRefCell<DOMString>,
}

impl<TH: TypeHolderTrait> HTMLDialogElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>) -> HTMLDialogElement<TH> {
        HTMLDialogElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document),
            return_value: DomRefCell::new(DOMString::new()),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLDialogElement<TH>> {
        Node::reflect_node(Box::new(HTMLDialogElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLDialogElementBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> HTMLDialogElementMethods for HTMLDialogElement<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-dialog-open
    make_bool_getter!(Open, "open");

    // https://html.spec.whatwg.org/multipage/#dom-dialog-open
    make_bool_setter!(SetOpen, "open");

    // https://html.spec.whatwg.org/multipage/#dom-dialog-returnvalue
    fn ReturnValue(&self) -> DOMString {
        let return_value = self.return_value.borrow();
        return_value.clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-dialog-returnvalue
    fn SetReturnValue(&self, return_value: DOMString) {
        *self.return_value.borrow_mut() = return_value;
    }

    // https://html.spec.whatwg.org/multipage/#dom-dialog-close
    fn Close(&self, return_value: Option<DOMString>) {
        let element = self.upcast::<Element<TH>>();
        let target = self.upcast::<EventTarget<TH>>();
        let win = window_from_node(self);

        // Step 1 & 2
        if element.remove_attribute(&ns!(), &local_name!("open")).is_none() {
            return;
        }

        // Step 3
        if let Some(new_value) = return_value {
            *self.return_value.borrow_mut() = new_value;
        }

        // TODO: Step 4 implement pending dialog stack removal

        // Step 5
        win.dom_manipulation_task_source().queue_simple_event(target, atom!("close"), &win);
    }
}
