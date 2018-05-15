/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::inheritance::Castable;
use dom::bindings::refcounted::Trusted;
use dom::event::{EventBubbles, EventCancelable, EventTask, SimpleEventTask};
use dom::eventtarget::EventTarget;
use dom::window::Window;
use msg::constellation_msg::PipelineId;
use script_runtime::{CommonScriptMsg, ScriptThreadEventCategory};
use script_thread::MainThreadScriptMsg;
use servo_atoms::Atom;
use std::fmt;
use std::result::Result;
use std::sync::mpsc::Sender;
use task::{TaskCanceller, TaskOnce};
use task_source::TaskSource;
use typeholder::TypeHolderTrait;

#[derive(Clone, JSTraceable)]
pub struct DOMManipulationTaskSource<TH: TypeHolderTrait>(pub Sender<MainThreadScriptMsg>, pub PipelineId);

impl<TH> fmt::Debug for DOMManipulationTaskSource<TH> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DOMManipulationTaskSource(...)")
    }
}

impl<TH> TaskSource for DOMManipulationTaskSource<TH> {
    fn queue_with_canceller<T>(
        &self,
        task: T,
        canceller: &TaskCanceller,
    ) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        let msg = MainThreadScriptMsg::Common(CommonScriptMsg::Task(
            ScriptThreadEventCategory::ScriptEvent,
            Box::new(canceller.wrap_task(task)),
            Some(self.1)
        ));
        self.0.send(msg).map_err(|_| ())
    }
}

impl<TH: TypeHolderTrait> DOMManipulationTaskSource<TH> {
    pub fn queue_event(&self,
                       target: &EventTarget<TH>,
                       name: Atom,
                       bubbles: EventBubbles,
                       cancelable: EventCancelable,
                       window: &Window<TH>) {
        let target = Trusted::new(target);
        let task = EventTask {
            target: target,
            name: name,
            bubbles: bubbles,
            cancelable: cancelable,
        };
        let _ = self.queue(task, window.upcast());
    }

    pub fn queue_simple_event(&self, target: &EventTarget<TH>, name: Atom, window: &Window<TH>) {
        let target = Trusted::new(target);
        let _ = self.queue(SimpleEventTask { target, name }, window.upcast());
    }
}
