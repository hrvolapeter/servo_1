/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMPointBinding::{DOMPointInit, DOMPointMethods, Wrap};
use dom::bindings::codegen::Bindings::DOMPointReadOnlyBinding::DOMPointReadOnlyMethods;
use dom::bindings::error::Fallible;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::dompointreadonly::{DOMPointReadOnly, DOMPointWriteMethods};
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use typeholder::TypeHolderTrait;

// http://dev.w3.org/fxtf/geometry/Overview.html#dompoint
#[dom_struct]
pub struct DOMPoint<TH: TypeHolderTrait + 'static> {
    point: DOMPointReadOnly<TH>,
}

impl<TH: TypeHolderTrait> DOMPoint<TH> {
    fn new_inherited(x: f64, y: f64, z: f64, w: f64) -> DOMPoint<TH> {
        DOMPoint {
            point: DOMPointReadOnly::new_inherited(x, y, z, w),
        }
    }

    pub fn new(global: &GlobalScope<TH>, x: f64, y: f64, z: f64, w: f64) -> DomRoot<DOMPoint<TH>> {
        reflect_dom_object(Box::new(DOMPoint::new_inherited(x, y, z, w)), global, Wrap)
    }

    pub fn Constructor(global: &GlobalScope<TH>,
                       x: f64,
                       y: f64,
                       z: f64,
                       w: f64)
                       -> Fallible<DomRoot<DOMPoint<TH>>> {
        Ok(DOMPoint::new(global, x, y, z, w))
    }

    pub fn new_from_init(global: &GlobalScope<TH>, p: &DOMPointInit) -> DomRoot<DOMPoint<TH>> {
        DOMPoint::new(global, p.x, p.y, p.z, p.w)
    }
}

impl<TH: TypeHolderTrait> DOMPointMethods for DOMPoint<TH> {
    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-x
    fn X(&self) -> f64 {
        self.point.X()
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-x
    fn SetX(&self, value: f64) {
        self.point.SetX(value);
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-y
    fn Y(&self) -> f64 {
        self.point.Y()
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-y
    fn SetY(&self, value: f64) {
        self.point.SetY(value);
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-z
    fn Z(&self) -> f64 {
        self.point.Z()
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-z
    fn SetZ(&self, value: f64) {
        self.point.SetZ(value);
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-w
    fn W(&self) -> f64 {
        self.point.W()
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-w
    fn SetW(&self, value: f64) {
        self.point.SetW(value);
    }
}
