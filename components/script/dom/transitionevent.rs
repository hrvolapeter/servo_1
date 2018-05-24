/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::TransitionEventBinding;
use dom::bindings::codegen::Bindings::TransitionEventBinding::{TransitionEventInit, TransitionEventMethods};
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::num::Finite;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::event::Event;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_atoms::Atom;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct TransitionEvent<TH: TypeHolderTrait + 'static> {
    event: Event<TH>,
    property_name: Atom,
    elapsed_time: Finite<f32>,
    pseudo_element: DOMString,
}

impl<TH: TypeHolderTrait> TransitionEvent<TH> {
    fn new_inherited(init: &TransitionEventInit) -> TransitionEvent<TH> {
        TransitionEvent {
            event: Event::new_inherited(),
            property_name: Atom::from(init.propertyName.clone()),
            elapsed_time: init.elapsedTime.clone(),
            pseudo_element: init.pseudoElement.clone()
        }
    }

    pub fn new(window: &Window<TH>,
               type_: Atom,
               init: &TransitionEventInit) -> DomRoot<TransitionEvent<TH>> {
        let ev = reflect_dom_object(Box::new(TransitionEvent::new_inherited(init)),
                                    window,
                                    TransitionEventBinding::Wrap);
        {
            let event = ev.upcast::<Event<TH>>();
            event.init_event(type_, init.parent.bubbles, init.parent.cancelable);
        }
        ev
    }

    pub fn Constructor(window: &Window<TH>,
                       type_: DOMString,
                       init: &TransitionEventInit) -> Fallible<DomRoot<TransitionEvent<TH>>, TH> {
        Ok(TransitionEvent::new(window, Atom::from(type_), init))
    }
}

impl<TH: TypeHolderTrait> TransitionEventMethods for TransitionEvent<TH> {
    // https://drafts.csswg.org/css-transitions/#Events-TransitionEvent-propertyName
    fn PropertyName(&self) -> DOMString {
        DOMString::from(&*self.property_name)
    }

    // https://drafts.csswg.org/css-transitions/#Events-TransitionEvent-elapsedTime
    fn ElapsedTime(&self) -> Finite<f32> {
        self.elapsed_time.clone()
    }

    // https://drafts.csswg.org/css-transitions/#Events-TransitionEvent-pseudoElement
    fn PseudoElement(&self) -> DOMString {
        self.pseudo_element.clone()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.upcast::<Event<TH>>().IsTrusted()
    }
}
