/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::PopStateEventBinding;
use dom::bindings::codegen::Bindings::PopStateEventBinding::PopStateEventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::bindings::trace::RootedTraceableBox;
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::window::Window;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSContext};
use js::jsval::JSVal;
use js::rust::HandleValue;
use servo_atoms::Atom;
use typeholder::TypeHolderTrait;

// https://html.spec.whatwg.org/multipage/#the-popstateevent-interface
#[dom_struct]
pub struct PopStateEvent<TH: TypeHolderTrait> {
    event: Event,
    #[ignore_malloc_size_of = "Defined in rust-mozjs"]
    state: Heap<JSVal>,
}

impl<TH: TypeHolderTrait> PopStateEvent<TH> {
    fn new_inherited() -> PopStateEvent<TH> {
        PopStateEvent {
            event: Event::new_inherited(),
            state: Heap::default(),
        }
    }

    pub fn new_uninitialized(window: &Window<TH>) -> DomRoot<PopStateEvent<TH>> {
        reflect_dom_object(Box::new(PopStateEvent::new_inherited()),
                           window,
                           PopStateEventBinding::Wrap)
    }

    pub fn new(window: &Window<TH>,
               type_: Atom,
               bubbles: bool,
               cancelable: bool,
               state: HandleValue)
               -> DomRoot<PopStateEvent<TH>> {
        let ev = PopStateEvent::new_uninitialized(window);
        ev.state.set(state.get());
        {
            let event = ev.upcast::<Event<TH>>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    pub fn Constructor(window: &Window<TH>,
                       type_: DOMString,
                       init: RootedTraceableBox<PopStateEventBinding::PopStateEventInit>)
                       -> Fallible<DomRoot<PopStateEvent<TH>>> {
        Ok(PopStateEvent::new(window,
                              Atom::from(type_),
                              init.parent.bubbles,
                              init.parent.cancelable,
                              init.state.handle()))
    }

    pub fn dispatch_jsval(target: &EventTarget<TH>,
                          window: &Window<TH>,
                          state: HandleValue) {
        let event = PopStateEvent::new(window, atom!("popstate"), true, false, state);
        event.upcast::<Event<TH>>().fire(target);
    }
}

impl<TH> PopStateEventMethods for PopStateEvent<TH> {
    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-popstateevent-state
    unsafe fn State(&self, _cx: *mut JSContext) -> JSVal {
        self.state.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
