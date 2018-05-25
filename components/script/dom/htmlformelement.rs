/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::HTMLButtonElementBinding::HTMLButtonElementMethods;
use dom::bindings::codegen::Bindings::HTMLFormControlsCollectionBinding::HTMLFormControlsCollectionMethods;
use dom::bindings::codegen::Bindings::HTMLFormElementBinding;
use dom::bindings::codegen::Bindings::HTMLFormElementBinding::HTMLFormElementMethods;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding::HTMLTextAreaElementMethods;
use dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::DomObject;
use dom::bindings::root::{Dom, DomOnceCell, DomRoot, RootedReference};
use dom::bindings::str::DOMString;
use dom::blob::Blob;
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::eventtarget::EventTarget;
use dom::file::File;
use dom::globalscope::GlobalScope;
use dom::htmlbuttonelement::HTMLButtonElement;
use dom::htmlcollection::CollectionFilter;
use dom::htmldatalistelement::HTMLDataListElement;
use dom::htmlelement::HTMLElement;
use dom::htmlfieldsetelement::HTMLFieldSetElement;
use dom::htmlformcontrolscollection::HTMLFormControlsCollection;
use dom::htmlimageelement::HTMLImageElement;
use dom::htmlinputelement::{HTMLInputElement, InputType};
use dom::htmllabelelement::HTMLLabelElement;
use dom::htmllegendelement::HTMLLegendElement;
use dom::htmlobjectelement::HTMLObjectElement;
use dom::htmloutputelement::HTMLOutputElement;
use dom::htmlselectelement::HTMLSelectElement;
use dom::htmltextareaelement::HTMLTextAreaElement;
use dom::node::{Node, NodeFlags, UnbindContext, VecPreOrderInsertionHelper};
use dom::node::{document_from_node, window_from_node};
use dom::validitystate::ValidationFlags;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use encoding_rs::{Encoding, UTF_8};
use html5ever::{LocalName, Prefix};
use hyper::header::{Charset, ContentDisposition, ContentType, DispositionParam, DispositionType};
use hyper::method::Method;
use script_thread::MainThreadScriptMsg;
use script_traits::LoadData;
use servo_rand::random;
use std::borrow::ToOwned;
use std::cell::Cell;
use style::attr::AttrValue;
use style::str::split_html_space_chars;
use task_source::TaskSource;
use url::UrlQuery;
use url::form_urlencoded::Serializer;
use typeholder::TypeHolderTrait;
use std::marker::PhantomData;

#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
pub struct GenerationId(u32);

#[dom_struct]
pub struct HTMLFormElement<TH: TypeHolderTrait + 'static> {
    htmlelement: HTMLElement<TH>,
    marked_for_reset: Cell<bool>,
    elements: DomOnceCell<HTMLFormControlsCollection<TH>>,
    generation_id: Cell<GenerationId>,
    controls: DomRefCell<Vec<Dom<Element<TH>>>>,
}

impl<TH: TypeHolderTrait> HTMLFormElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>) -> HTMLFormElement<TH> {
        HTMLFormElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            marked_for_reset: Cell::new(false),
            elements: Default::default(),
            generation_id: Cell::new(GenerationId(0)),
            controls: DomRefCell::new(Vec::new()),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLFormElement<TH>> {
        Node::reflect_node(Box::new(HTMLFormElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLFormElementBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> HTMLFormElementMethods<TH> for HTMLFormElement<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-form-acceptcharset
    make_getter!(AcceptCharset, "accept-charset");

    // https://html.spec.whatwg.org/multipage/#dom-form-acceptcharset
    make_setter!(SetAcceptCharset, "accept-charset");

    // https://html.spec.whatwg.org/multipage/#dom-fs-action
    make_form_action_getter!(Action, "action");

    // https://html.spec.whatwg.org/multipage/#dom-fs-action
    make_setter!(SetAction, "action");

    // https://html.spec.whatwg.org/multipage/#dom-form-autocomplete
    make_enumerated_getter!(Autocomplete, "autocomplete", "on", "off");

    // https://html.spec.whatwg.org/multipage/#dom-form-autocomplete
    make_setter!(SetAutocomplete, "autocomplete");

    // https://html.spec.whatwg.org/multipage/#dom-fs-enctype
    make_enumerated_getter!(Enctype,
                            "enctype",
                            "application/x-www-form-urlencoded",
                            "text/plain" | "multipart/form-data");

    // https://html.spec.whatwg.org/multipage/#dom-fs-enctype
    make_setter!(SetEnctype, "enctype");

    // https://html.spec.whatwg.org/multipage/#dom-fs-encoding
    fn Encoding(&self) -> DOMString {
        self.Enctype()
    }

    // https://html.spec.whatwg.org/multipage/#dom-fs-encoding
    fn SetEncoding(&self, value: DOMString) {
        self.SetEnctype(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-fs-method
    make_enumerated_getter!(Method, "method", "get", "post" | "dialog");

    // https://html.spec.whatwg.org/multipage/#dom-fs-method
    make_setter!(SetMethod, "method");

    // https://html.spec.whatwg.org/multipage/#dom-form-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-form-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-fs-novalidate
    make_bool_getter!(NoValidate, "novalidate");

    // https://html.spec.whatwg.org/multipage/#dom-fs-novalidate
    make_bool_setter!(SetNoValidate, "novalidate");

    // https://html.spec.whatwg.org/multipage/#dom-fs-target
    make_getter!(Target, "target");

    // https://html.spec.whatwg.org/multipage/#dom-fs-target
    make_setter!(SetTarget, "target");

    // https://html.spec.whatwg.org/multipage/#the-form-element:concept-form-submit
    fn Submit(&self) {
        self.submit(SubmittedFrom::FromForm, FormSubmitter::FormElement(self));
    }

    // https://html.spec.whatwg.org/multipage/#dom-form-reset
    fn Reset(&self) {
        self.reset(ResetFrom::FromForm);
    }

    // https://html.spec.whatwg.org/multipage/#dom-form-elements
    fn Elements(&self) -> DomRoot<HTMLFormControlsCollection<TH>> {
        #[derive(JSTraceable, MallocSizeOf)]
        struct ElementsFilter<THH: TypeHolderTrait + 'static> {
            form: DomRoot<HTMLFormElement<THH>>,
            _p: PhantomData<THH>,
        }
        impl<THH: TypeHolderTrait> CollectionFilter<THH> for ElementsFilter<THH> {
            fn filter<'a>(&self, elem: &'a Element<THH>, _root: &'a Node<THH>) -> bool {
                let form_owner = match elem.upcast::<Node<THH>>().type_id() {
                    NodeTypeId::Element(ElementTypeId::HTMLElement(t)) => {
                        match t {
                            HTMLElementTypeId::HTMLButtonElement => {
                                elem.downcast::<HTMLButtonElement<THH>>().unwrap().form_owner()
                            }
                            HTMLElementTypeId::HTMLFieldSetElement => {
                                elem.downcast::<HTMLFieldSetElement<THH>>().unwrap().form_owner()
                            }
                            HTMLElementTypeId::HTMLInputElement => {
                                let input_elem = elem.downcast::<HTMLInputElement<THH>>().unwrap();
                                if input_elem.input_type() == InputType::Image {
                                    return false;
                                }
                                input_elem.form_owner()
                            }
                            HTMLElementTypeId::HTMLObjectElement => {
                                elem.downcast::<HTMLObjectElement<THH>>().unwrap().form_owner()
                            }
                            HTMLElementTypeId::HTMLOutputElement => {
                                elem.downcast::<HTMLOutputElement<THH>>().unwrap().form_owner()
                            }
                            HTMLElementTypeId::HTMLSelectElement => {
                                elem.downcast::<HTMLSelectElement<THH>>().unwrap().form_owner()
                            }
                            HTMLElementTypeId::HTMLTextAreaElement => {
                                elem.downcast::<HTMLTextAreaElement<THH>>().unwrap().form_owner()
                            }
                            _ => {
                                debug_assert!(!elem.downcast::<HTMLElement<THH>>().unwrap().is_listed_element() ||
                                              elem.local_name() == &local_name!("keygen"));
                                return false;
                            }
                        }
                    }
                    _ => return false,
                };

                match form_owner {
                    Some(form_owner) => form_owner == self.form,
                    None => false,
                }
            }
        }
        DomRoot::from_ref(self.elements.init_once(|| {
            let filter = Box::new(ElementsFilter { form: DomRoot::from_ref(self) });
            let window = window_from_node(self);
            HTMLFormControlsCollection::new(&window, self.upcast(), filter)
        }))
    }

    // https://html.spec.whatwg.org/multipage/#dom-form-length
    fn Length(&self) -> u32 {
        self.Elements().Length() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-form-item
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<Element<TH>>> {
        let elements = self.Elements();
        elements.IndexedGetter(index)
    }
}

#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
pub enum SubmittedFrom {
    FromForm,
    NotFromForm
}

#[derive(Clone, Copy, MallocSizeOf)]
pub enum ResetFrom {
    FromForm,
    NotFromForm
}


impl<TH: TypeHolderTrait> HTMLFormElement<TH> {
    // https://html.spec.whatwg.org/multipage/#picking-an-encoding-for-the-form
    fn pick_encoding(&self) -> &'static Encoding {
        // Step 2
        if self.upcast::<Element<TH>>().has_attribute(&local_name!("accept-charset")) {
            // Substep 1
            let input = self.upcast::<Element<TH>>().get_string_attribute(&local_name!("accept-charset"));

            // Substep 2, 3, 4
            let mut candidate_encodings = split_html_space_chars(&*input)
                .filter_map(|c| Encoding::for_label(c.as_bytes()));

            // Substep 5, 6
            return candidate_encodings.next().unwrap_or(UTF_8);
        }

        // Step 1, 3
        document_from_node(self).encoding()
    }

    // https://html.spec.whatwg.org/multipage/#text/plain-encoding-algorithm
    fn encode_plaintext(&self, form_data: &mut Vec<FormDatum<TH>>) -> String {
        // Step 1
        let mut result = String::new();

        // Step 2
        let encoding = self.pick_encoding();

        // Step 3
        let charset = encoding.name();

        for entry in form_data.iter_mut() {
            // Step 4, 5
            let value = entry.replace_value(charset);

            // Step 6
            result.push_str(&*format!("{}={}\r\n", entry.name, value));
        }

        // Step 7
        result
    }

    /// [Form submission](https://html.spec.whatwg.org/multipage/#concept-form-submit)
    pub fn submit(&self, submit_method_flag: SubmittedFrom, submitter: FormSubmitter<TH>) {
        // Step 1
        let doc = document_from_node(self);
        let base = doc.base_url();
        // TODO: Handle browsing contexts (Step 2, 3)
        // Step 4
        if submit_method_flag == SubmittedFrom::NotFromForm &&
           !submitter.no_validate(self)
        {
            if self.interactive_validation().is_err() {
                // TODO: Implement event handlers on all form control elements
                self.upcast::<EventTarget<TH>>().fire_event(atom!("invalid"));
                return;
            }
        }
        // Step 5
        if submit_method_flag == SubmittedFrom::NotFromForm {
            let event = self.upcast::<EventTarget<TH>>()
                .fire_bubbling_cancelable_event(atom!("submit"));
            if event.DefaultPrevented() {
                return;
            }
        }
        // Step 6
        let mut form_data = self.get_form_dataset(Some(submitter));

        // Step 7
        let encoding = self.pick_encoding();

        // Step 8
        let mut action = submitter.action();

        // Step 9
        if action.is_empty() {
            action = DOMString::from(base.as_str());
        }
        // Step 10-11
        let action_components = match base.join(&action) {
            Ok(url) => url,
            Err(_) => return
        };
        // Step 12-15
        let scheme = action_components.scheme().to_owned();
        let enctype = submitter.enctype();
        let method = submitter.method();
        let _target = submitter.target();
        // TODO: Handle browsing contexts, partially loaded documents (step 16-17)

        let mut load_data = LoadData::new(action_components, None, doc.get_referrer_policy(), Some(doc.url()));

        // Step 18
        match (&*scheme, method) {
            (_, FormMethod::FormDialog) => {
                // TODO: Submit dialog
                // https://html.spec.whatwg.org/multipage/#submit-dialog
            }
            // https://html.spec.whatwg.org/multipage/#submit-mutate-action
            ("http", FormMethod::FormGet) | ("https", FormMethod::FormGet) | ("data", FormMethod::FormGet) => {
                load_data.headers.set(ContentType::form_url_encoded());
                self.mutate_action_url(&mut form_data, load_data, encoding);
            }
            // https://html.spec.whatwg.org/multipage/#submit-body
            ("http", FormMethod::FormPost) | ("https", FormMethod::FormPost) => {
                load_data.method = Method::Post;
                self.submit_entity_body(&mut form_data, load_data, enctype, encoding);
            }
            // https://html.spec.whatwg.org/multipage/#submit-get-action
            ("file", _) | ("about", _) | ("data", FormMethod::FormPost) |
            ("ftp", _) | ("javascript", _) => {
                self.plan_to_navigate(load_data);
            }
            ("mailto", FormMethod::FormPost) => {
                // TODO: Mail as body
                // https://html.spec.whatwg.org/multipage/#submit-mailto-body
            }
            ("mailto", FormMethod::FormGet) => {
                // TODO: Mail with headers
                // https://html.spec.whatwg.org/multipage/#submit-mailto-headers
            }
            _ => return,
        }
    }

    // https://html.spec.whatwg.org/multipage/#submit-mutate-action
    fn mutate_action_url(&self, form_data: &mut Vec<FormDatum<TH>>, mut load_data: LoadData, encoding: &'static Encoding) {
        let charset = encoding.name();

        self.set_encoding_override(load_data.url.as_mut_url().query_pairs_mut())
            .clear()
            .extend_pairs(form_data.into_iter()
                                    .map(|field| (field.name.clone(), field.replace_value(charset))));

        self.plan_to_navigate(load_data);
    }

    // https://html.spec.whatwg.org/multipage/#submit-body
    fn submit_entity_body(&self, form_data: &mut Vec<FormDatum<TH>>, mut load_data: LoadData,
                          enctype: FormEncType, encoding: &'static Encoding) {
        let boundary = generate_boundary();
        let bytes = match enctype {
            FormEncType::UrlEncoded => {
                let charset = encoding.name();
                load_data.headers.set(ContentType::form_url_encoded());

                self.set_encoding_override(load_data.url.as_mut_url().query_pairs_mut())
                    .clear()
                    .extend_pairs(form_data.into_iter()
                    .map(|field| (field.name.clone(), field.replace_value(charset))));

                load_data.url.query().unwrap_or("").to_string().into_bytes()
            }
            FormEncType::FormDataEncoded => {
                let mime = mime!(Multipart / FormData; Boundary =(&boundary));
                load_data.headers.set(ContentType(mime));
                encode_multipart_form_data(form_data, boundary, encoding)
            }
            FormEncType::TextPlainEncoded => {
                load_data.headers.set(ContentType(mime!(Text / Plain)));
                self.encode_plaintext(form_data).into_bytes()
            }
        };

        load_data.data = Some(bytes);
        self.plan_to_navigate(load_data);
    }

    fn set_encoding_override<'a>(&self, mut serializer: Serializer<UrlQuery<'a>>)
                                 -> Serializer<UrlQuery<'a>> {
        let encoding = self.pick_encoding();
        serializer.custom_encoding_override(move |s| encoding.encode(s).0);
        serializer
    }

    /// [Planned navigation](https://html.spec.whatwg.org/multipage/#planned-navigation)
    fn plan_to_navigate(&self, load_data: LoadData) {
        let window = window_from_node(self);

        // Step 1
        // Each planned navigation task is tagged with a generation ID, and
        // before the task is handled, it first checks whether the HTMLFormElement's
        // generation ID is the same as its own generation ID.
        let generation_id = GenerationId(self.generation_id.get().0 + 1);
        self.generation_id.set(generation_id);

        // Step 2.
        let pipeline_id = window.upcast::<GlobalScope<TH>>().pipeline_id();
        let script_chan = window.main_thread_script_chan().clone();
        let this = Trusted::new(self);
        let task = task!(navigate_to_form_planned_navigation: move || {
            if generation_id != this.root().generation_id.get() {
                return;
            }
            script_chan.send(MainThreadScriptMsg::Navigate(
                pipeline_id,
                load_data,
                false,
            )).unwrap();
        });

        // Step 3.
        window.dom_manipulation_task_source().queue(task, window.upcast()).unwrap();
    }

    /// Interactively validate the constraints of form elements
    /// <https://html.spec.whatwg.org/multipage/#interactively-validate-the-constraints>
    fn interactive_validation(&self) -> Result<(), ()> {
        // Step 1-3
        let _unhandled_invalid_controls = match self.static_validation() {
            Ok(()) => return Ok(()),
            Err(err) => err
        };
        // TODO: Report the problems with the constraints of at least one of
        //       the elements given in unhandled invalid controls to the user
        // Step 4
        Err(())
    }

    /// Statitically validate the constraints of form elements
    /// <https://html.spec.whatwg.org/multipage/#statically-validate-the-constraints>
    fn static_validation(&self) -> Result<(), Vec<FormSubmittableElement<TH>>> {
        let node = self.upcast::<Node<TH>>();
        // FIXME(#3553): This is an incorrect way of getting controls owned by the
        //               form, refactor this when html5ever's form owner PR lands
        // Step 1-3
        let invalid_controls = node.traverse_preorder().filter_map(|field| {
            if let Some(el) = field.downcast::<Element<TH>>() {
                if el.disabled_state() {
                    None
                } else {
                    let validatable = match el.as_maybe_validatable() {
                        Some(v) => v,
                        None => return None
                    };
                    if !validatable.is_instance_validatable() {
                        None
                    } else if validatable.validate(ValidationFlags::empty()) {
                        None
                    } else {
                        Some(FormSubmittableElement::from_element(&el))
                    }
                }
            } else {
                None
            }
        }).collect::<Vec<FormSubmittableElement<TH>>>();
        // Step 4
        if invalid_controls.is_empty() { return Ok(()); }
        // Step 5-6
        let unhandled_invalid_controls = invalid_controls.into_iter().filter_map(|field| {
            let event = field.as_event_target()
                .fire_cancelable_event(atom!("invalid"));
            if !event.DefaultPrevented() { return Some(field); }
            None
        }).collect::<Vec<FormSubmittableElement<TH>>>();
        // Step 7
        Err(unhandled_invalid_controls)
    }

    /// <https://html.spec.whatwg.org/multipage/#constructing-the-form-data-set>
    /// Steps range from 1 to 3
    fn get_unclean_dataset(&self, submitter: Option<FormSubmitter<TH>>) -> Vec<FormDatum<TH>> {
        let controls = self.controls.borrow();
        let mut data_set = Vec::new();
        for child in controls.iter() {
            // Step 3.1: The field element is disabled.
            if child.disabled_state() {
                continue;
            }
            let child = child.upcast::<Node<TH>>();

            // Step 3.1: The field element has a datalist element ancestor.
            if child.ancestors()
                    .any(|a| DomRoot::downcast::<HTMLDataListElement<TH>>(a).is_some()) {
                continue;
            }
            if let NodeTypeId::Element(ElementTypeId::HTMLElement(element)) = child.type_id() {
                match element {
                    HTMLElementTypeId::HTMLInputElement => {
                        let input = child.downcast::<HTMLInputElement<TH>>().unwrap();

                        data_set.append(&mut input.form_datums(submitter));
                    }
                    HTMLElementTypeId::HTMLButtonElement => {
                        let button = child.downcast::<HTMLButtonElement<TH>>().unwrap();
                        if let Some(datum) = button.form_datum(submitter) {
                            data_set.push(datum);
                        }
                    }
                    HTMLElementTypeId::HTMLObjectElement => {
                        // Unimplemented
                        ()
                    }
                    HTMLElementTypeId::HTMLSelectElement => {
                        let select = child.downcast::<HTMLSelectElement<TH>>().unwrap();
                        select.push_form_data(&mut data_set);
                    }
                    HTMLElementTypeId::HTMLTextAreaElement => {
                        let textarea = child.downcast::<HTMLTextAreaElement<TH>>().unwrap();
                        let name = textarea.Name();
                        if !name.is_empty() {
                            data_set.push(FormDatum {
                                ty: textarea.Type(),
                                name: name,
                                value: FormDatumValue::String(textarea.Value())
                            });
                        }
                    }
                    _ => ()
                }
            }
        }
        data_set
        // TODO: Handle `dirnames` (needs directionality support)
        //       https://html.spec.whatwg.org/multipage/#the-directionality
    }

    /// <https://html.spec.whatwg.org/multipage/#constructing-the-form-data-set>
    pub fn get_form_dataset(&self, submitter: Option<FormSubmitter<TH>>) -> Vec<FormDatum<TH>> {
        fn clean_crlf(s: &str) -> DOMString {
            // Step 4
            let mut buf = "".to_owned();
            let mut prev = ' ';
            for ch in s.chars() {
                match ch {
                    '\n' if prev != '\r' => {
                        buf.push('\r');
                        buf.push('\n');
                    },
                    '\n' => {
                        buf.push('\n');
                    },
                    // This character isn't LF but is
                    // preceded by CR
                    _ if prev == '\r' => {
                        buf.push('\r');
                        buf.push('\n');
                        buf.push(ch);
                    },
                    _ => buf.push(ch)
                };
                prev = ch;
            }
            // In case the last character was CR
            if prev == '\r' {
                buf.push('\n');
            }
            DOMString::from(buf)
        }

        // Step 1-3
        let mut ret = self.get_unclean_dataset(submitter);
        // Step 4
        for datum in &mut ret {
            match &*datum.ty {
                "file" | "textarea" => (), // TODO
                _ => {
                    datum.name = clean_crlf(&datum.name);
                    datum.value = FormDatumValue::String(clean_crlf(match datum.value {
                        FormDatumValue::String(ref s) => s,
                        FormDatumValue::File(_) => unreachable!()
                    }));
                }
            }
        };
        // Step 5
        ret
    }

    pub fn reset(&self, _reset_method_flag: ResetFrom) {
        // https://html.spec.whatwg.org/multipage/#locked-for-reset
        if self.marked_for_reset.get() {
            return;
        } else {
            self.marked_for_reset.set(true);
        }

        let event = self.upcast::<EventTarget<TH>>()
            .fire_bubbling_cancelable_event(atom!("reset"));
        if event.DefaultPrevented() {
            return;
        }

        let controls = self.controls.borrow();
        for child in controls.iter() {
            let child = child.upcast::<Node<TH>>();

            match child.type_id() {
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                    child.downcast::<HTMLInputElement<TH>>().unwrap().reset();
                }
                // TODO HTMLKeygenElement unimplemented
                //NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLKeygenElement)) => {
                //    // Unimplemented
                //    {}
                //}
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
                    child.downcast::<HTMLSelectElement<TH>>().unwrap().reset();
                }
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
                    child.downcast::<HTMLTextAreaElement<TH>>().unwrap().reset();
                }
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOutputElement)) => {
                    // Unimplemented
                }
                _ => {}
            }
        }
        self.marked_for_reset.set(false);
    }

    fn add_control<T: ?Sized + FormControl<TH>>(&self, control: &T) {
        let root = self.upcast::<Element<TH>>().root_element();
        let root = root.r().upcast::<Node<TH>>();

        let mut controls = self.controls.borrow_mut();
        controls.insert_pre_order(control.to_element(), root);
    }

    fn remove_control<T: ?Sized + FormControl<TH>>(&self, control: &T) {
        let control = control.to_element();
        let mut controls = self.controls.borrow_mut();
        controls.iter().position(|c| c.r() == control)
                       .map(|idx| controls.remove(idx));
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
pub enum FormDatumValue<TH: TypeHolderTrait + 'static> {
    #[allow(dead_code)]
    File(DomRoot<File<TH>>),
    String(DOMString)
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
pub struct FormDatum<TH: TypeHolderTrait + 'static> {
    pub ty: DOMString,
    pub name: DOMString,
    pub value: FormDatumValue<TH>
}

impl<TH: TypeHolderTrait> FormDatum<TH> {
    pub fn replace_value(&self, charset: &str) -> String {
        if self.name == "_charset_" && self.ty == "hidden" {
            return charset.to_string();
        }

        match self.value {
            FormDatumValue::File(ref f) => String::from(f.name().clone()),
            FormDatumValue::String(ref s) => String::from(s.clone()),
        }
    }
}

#[derive(Clone, Copy, MallocSizeOf)]
pub enum FormEncType {
    TextPlainEncoded,
    UrlEncoded,
    FormDataEncoded
}

#[derive(Clone, Copy, MallocSizeOf)]
pub enum FormMethod {
    FormGet,
    FormPost,
    FormDialog
}

#[derive(MallocSizeOf)]
#[allow(dead_code)]
pub enum FormSubmittableElement<TH: TypeHolderTrait + 'static> {
    ButtonElement(DomRoot<HTMLButtonElement<TH>>),
    InputElement(DomRoot<HTMLInputElement<TH>>),
    // TODO: HTMLKeygenElement unimplemented
    // KeygenElement(&'a HTMLKeygenElement),
    ObjectElement(DomRoot<HTMLObjectElement<TH>>),
    SelectElement(DomRoot<HTMLSelectElement<TH>>),
    TextAreaElement(DomRoot<HTMLTextAreaElement<TH>>),
}

impl<TH: TypeHolderTrait> FormSubmittableElement<TH> {
    fn as_event_target(&self) -> &EventTarget<TH> {
        match *self {
            FormSubmittableElement::ButtonElement(ref button) => button.upcast(),
            FormSubmittableElement::InputElement(ref input) => input.upcast(),
            FormSubmittableElement::ObjectElement(ref object) => object.upcast(),
            FormSubmittableElement::SelectElement(ref select) => select.upcast(),
            FormSubmittableElement::TextAreaElement(ref textarea) => textarea.upcast()
        }
    }

    fn from_element(element: &Element<TH>) -> FormSubmittableElement<TH> {
        if let Some(input) = element.downcast::<HTMLInputElement<TH>>() {
            FormSubmittableElement::InputElement(DomRoot::from_ref(&input))
        }
        else if let Some(input) = element.downcast::<HTMLButtonElement<TH>>() {
            FormSubmittableElement::ButtonElement(DomRoot::from_ref(&input))
        }
        else if let Some(input) = element.downcast::<HTMLObjectElement<TH>>() {
            FormSubmittableElement::ObjectElement(DomRoot::from_ref(&input))
        }
        else if let Some(input) = element.downcast::<HTMLSelectElement<TH>>() {
            FormSubmittableElement::SelectElement(DomRoot::from_ref(&input))
        }
        else if let Some(input) = element.downcast::<HTMLTextAreaElement<TH>>() {
            FormSubmittableElement::TextAreaElement(DomRoot::from_ref(&input))
        } else {
            unreachable!()
        }
    }
}

#[derive(Clone, Copy, MallocSizeOf)]
pub enum FormSubmitter<'a, TH: TypeHolderTrait + 'static> {
    FormElement(&'a HTMLFormElement<TH>),
    InputElement(&'a HTMLInputElement<TH>),
    ButtonElement(&'a HTMLButtonElement<TH>)
    // TODO: image submit, etc etc
}

impl<'a, TH: TypeHolderTrait> FormSubmitter<'a, TH> {
    fn action(&self) -> DOMString {
        match *self {
            FormSubmitter::FormElement(form) => form.Action(),
            FormSubmitter::InputElement(input_element) => {
                input_element.get_form_attribute(&local_name!("formaction"),
                                                 |i| i.FormAction(),
                                                 |f| f.Action())
            },
            FormSubmitter::ButtonElement(button_element) => {
                button_element.get_form_attribute(&local_name!("formaction"),
                                                  |i| i.FormAction(),
                                                  |f| f.Action())
            }
        }
    }

    fn enctype(&self) -> FormEncType {
        let attr = match *self {
            FormSubmitter::FormElement(form) => form.Enctype(),
            FormSubmitter::InputElement(input_element) => {
                input_element.get_form_attribute(&local_name!("formenctype"),
                                                 |i| i.FormEnctype(),
                                                 |f| f.Enctype())
            },
            FormSubmitter::ButtonElement(button_element) => {
                button_element.get_form_attribute(&local_name!("formenctype"),
                                                  |i| i.FormEnctype(),
                                                  |f| f.Enctype())
            }
        };
        match &*attr {
            "multipart/form-data" => FormEncType::FormDataEncoded,
            "text/plain" => FormEncType::TextPlainEncoded,
            // https://html.spec.whatwg.org/multipage/#attr-fs-enctype
            // urlencoded is the default
            _ => FormEncType::UrlEncoded
        }
    }

    fn method(&self) -> FormMethod {
        let attr = match *self {
            FormSubmitter::FormElement(form) => form.Method(),
            FormSubmitter::InputElement(input_element) => {
                input_element.get_form_attribute(&local_name!("formmethod"),
                                                 |i| i.FormMethod(),
                                                 |f| f.Method())
            },
            FormSubmitter::ButtonElement(button_element) => {
                button_element.get_form_attribute(&local_name!("formmethod"),
                                                  |i| i.FormMethod(),
                                                  |f| f.Method())
            }
        };
        match &*attr {
            "dialog" => FormMethod::FormDialog,
            "post" => FormMethod::FormPost,
            _ => FormMethod::FormGet
        }
    }

    fn target(&self) -> DOMString {
        match *self {
            FormSubmitter::FormElement(form) => form.Target(),
            FormSubmitter::InputElement(input_element) => {
                input_element.get_form_attribute(&local_name!("formtarget"),
                                                 |i| i.FormTarget(),
                                                 |f| f.Target())
            },
            FormSubmitter::ButtonElement(button_element) => {
                button_element.get_form_attribute(&local_name!("formtarget"),
                                                  |i| i.FormTarget(),
                                                  |f| f.Target())
            }
        }
    }

    fn no_validate(&self, _form_owner: &HTMLFormElement<TH>) -> bool {
        match *self {
            FormSubmitter::FormElement(form) => form.NoValidate(),
            FormSubmitter::InputElement(input_element) => {
                input_element.get_form_boolean_attribute(&local_name!("formnovalidate"),
                                                 |i| i.FormNoValidate(),
                                                 |f| f.NoValidate())
            }
            FormSubmitter::ButtonElement(button_element) => {
                button_element.get_form_boolean_attribute(&local_name!("formnovalidate"),
                                                  |i| i.FormNoValidate(),
                                                  |f| f.NoValidate())
            }
        }
    }
}

pub trait FormControl<TH: TypeHolderTrait>: DomObject {
    fn form_owner(&self) -> Option<DomRoot<HTMLFormElement<TH>>>;

    fn set_form_owner(&self, form: Option<&HTMLFormElement<TH>>);

    fn to_element<'a>(&'a self) -> &'a Element<TH>;

    fn is_listed(&self) -> bool {
        true
    }

    // https://html.spec.whatwg.org/multipage/#create-an-element-for-the-token
    // Part of step 12.
    // '..suppress the running of the reset the form owner algorithm
    // when the parser subsequently attempts to insert the element..'
    fn set_form_owner_from_parser(&self, form: &HTMLFormElement<TH>) {
        let elem = self.to_element();
        let node = elem.upcast::<Node<TH>>();
        node.set_flag(NodeFlags::PARSER_ASSOCIATED_FORM_OWNER, true);
        form.add_control(self);
        self.set_form_owner(Some(form));
    }

    // https://html.spec.whatwg.org/multipage/#reset-the-form-owner
    fn reset_form_owner(&self) {
        let elem = self.to_element();
        let node = elem.upcast::<Node<TH>>();
        let old_owner = self.form_owner();
        let has_form_id = elem.has_attribute(&local_name!("form"));
        let nearest_form_ancestor = node.ancestors()
                                        .filter_map(DomRoot::downcast::<HTMLFormElement<TH>>)
                                        .next();

        // Step 1
        if old_owner.is_some() && !(self.is_listed() && has_form_id) {
            if nearest_form_ancestor == old_owner {
                return;
            }
        }

        let new_owner = if self.is_listed() && has_form_id && elem.is_connected() {
            // Step 3
            let doc = document_from_node(node);
            let form_id = elem.get_string_attribute(&local_name!("form"));
            doc.GetElementById(form_id).and_then(DomRoot::downcast::<HTMLFormElement<TH>>)
        } else {
            // Step 4
            nearest_form_ancestor
        };

        if old_owner != new_owner {
            if let Some(o) = old_owner {
                o.remove_control(self);
            }
            let new_owner = new_owner.as_ref().map(|o| {
                o.add_control(self);
                o.r()
            });
            self.set_form_owner(new_owner);
        }
    }

    // https://html.spec.whatwg.org/multipage/#association-of-controls-and-forms
    fn form_attribute_mutated(&self, mutation: AttributeMutation<TH>) {
        match mutation {
            AttributeMutation::Set(_) => {
                self.register_if_necessary();
            },
            AttributeMutation::Removed => {
                self.unregister_if_necessary();
            },
        }

        self.reset_form_owner();
    }

    // https://html.spec.whatwg.org/multipage/#association-of-controls-and-forms
    fn register_if_necessary(&self) {
        let elem = self.to_element();
        let form_id = elem.get_string_attribute(&local_name!("form"));
        let node = elem.upcast::<Node<TH>>();

        if self.is_listed() && !form_id.is_empty() && node.is_in_doc() {
            let doc = document_from_node(node);
            doc.register_form_id_listener(form_id, self);
        }
    }

    fn unregister_if_necessary(&self) {
        let elem = self.to_element();
        let form_id = elem.get_string_attribute(&local_name!("form"));

        if self.is_listed() && !form_id.is_empty() {
            let doc = document_from_node(elem.upcast::<Node<TH>>());
            doc.unregister_form_id_listener(form_id, self);
        }
    }

    // https://html.spec.whatwg.org/multipage/#association-of-controls-and-forms
    fn bind_form_control_to_tree(&self) {
        let elem = self.to_element();
        let node = elem.upcast::<Node<TH>>();

        // https://html.spec.whatwg.org/multipage/#create-an-element-for-the-token
        // Part of step 12.
        // '..suppress the running of the reset the form owner algorithm
        // when the parser subsequently attempts to insert the element..'
        let must_skip_reset = node.get_flag(NodeFlags::PARSER_ASSOCIATED_FORM_OWNER);
        node.set_flag(NodeFlags::PARSER_ASSOCIATED_FORM_OWNER, false);

        if !must_skip_reset {
            self.form_attribute_mutated(AttributeMutation::Set(None));
        }
    }

    // https://html.spec.whatwg.org/multipage/#association-of-controls-and-forms
    fn unbind_form_control_from_tree(&self) {
        let elem = self.to_element();
        let has_form_attr = elem.has_attribute(&local_name!("form"));
        let same_subtree = self.form_owner().map_or(true, |form| {
            elem.is_in_same_home_subtree(&*form)
        });

        self.unregister_if_necessary();

        // Since this control has been unregistered from the id->listener map
        // in the previous step, reset_form_owner will not be invoked on it
        // when the form owner element is unbound (i.e it is in the same
        // subtree) if it appears later in the tree order. Hence invoke
        // reset from here if this control has the form attribute set.
        if !same_subtree || (self.is_listed() && has_form_attr) {
            self.reset_form_owner();
        }
    }

    fn get_form_attribute<InputFn, OwnerFn>(&self,
                                            attr: &LocalName,
                                            input: InputFn,
                                            owner: OwnerFn)
                                            -> DOMString
        where InputFn: Fn(&Self) -> DOMString,
              OwnerFn: Fn(&HTMLFormElement<TH>) -> DOMString, Self: Sized
    {
        if self.to_element().has_attribute(attr) {
            input(self)
        } else {
            self.form_owner().map_or(DOMString::new(), |t| owner(&t))
        }
    }

    fn get_form_boolean_attribute<InputFn, OwnerFn>(&self,
                                            attr: &LocalName,
                                            input: InputFn,
                                            owner: OwnerFn)
                                            -> bool
        where InputFn: Fn(&Self) -> bool,
              OwnerFn: Fn(&HTMLFormElement<TH>) -> bool, Self: Sized
    {
        if self.to_element().has_attribute(attr) {
            input(self)
        } else {
            self.form_owner().map_or(false, |t| owner(&t))
        }
    }

    // XXXKiChjang: Implement these on inheritors
    // fn candidate_for_validation(&self) -> bool;
    // fn satisfies_constraints(&self) -> bool;
}

impl<TH: TypeHolderTrait> VirtualMethods<TH> for HTMLFormElement<TH> {
    fn super_type(&self) -> Option<&VirtualMethods<TH>> {
        Some(self.upcast::<HTMLElement<TH>>() as &VirtualMethods<TH>)
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("name") => AttrValue::from_atomic(value.into()),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }

    fn unbind_from_tree(&self, context: &UnbindContext<TH>) {
        self.super_type().unwrap().unbind_from_tree(context);

        // Collect the controls to reset because reset_form_owner
        // will mutably borrow self.controls
        rooted_vec!(let mut to_reset);
        to_reset.extend(self.controls.borrow().iter()
                        .filter(|c| !c.is_in_same_home_subtree(self))
                        .map(|c| c.clone()));

        for control in to_reset.iter() {
            control.as_maybe_form_control()
                       .expect("Element must be a form control")
                       .reset_form_owner();
        }
    }
}

pub trait FormControlElementHelpers<TH: TypeHolderTrait> {
    fn as_maybe_form_control<'a>(&'a self) -> Option<&'a FormControl<TH, TypeHolder=TH>>;
}

impl<TH: TypeHolderTrait> FormControlElementHelpers<TH> for Element<TH> {
    fn as_maybe_form_control<'a>(&'a self) -> Option<&'a FormControl<TH, TypeHolder=TH>> {
        let node = self.upcast::<Node<TH>>();

        match node.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
                Some(self.downcast::<HTMLButtonElement<TH>>().unwrap() as &FormControl<TH>)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFieldSetElement)) => {
                Some(self.downcast::<HTMLFieldSetElement<TH>>().unwrap() as &FormControl<TH>)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLImageElement)) => {
                Some(self.downcast::<HTMLImageElement<TH>>().unwrap() as &FormControl<TH>)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                Some(self.downcast::<HTMLInputElement<TH>>().unwrap() as &FormControl<TH>)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLabelElement)) => {
                Some(self.downcast::<HTMLLabelElement<TH>>().unwrap() as &FormControl<TH>)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLegendElement)) => {
                Some(self.downcast::<HTMLLegendElement<TH>>().unwrap() as &FormControl<TH>)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)) => {
                Some(self.downcast::<HTMLObjectElement<TH>>().unwrap() as &FormControl<TH>)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOutputElement)) => {
                Some(self.downcast::<HTMLOutputElement<TH>>().unwrap() as &FormControl<TH>)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
                Some(self.downcast::<HTMLSelectElement<TH>>().unwrap() as &FormControl<TH>)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
                Some(self.downcast::<HTMLTextAreaElement<TH>>().unwrap() as &FormControl<TH>)
            },
            _ => {
                None
            }
        }
    }
}

// https://html.spec.whatwg.org/multipage/#multipart/form-data-encoding-algorithm
pub fn encode_multipart_form_data<TH: TypeHolderTrait>(form_data: &mut Vec<FormDatum<TH>>,
                                  boundary: String, encoding: &'static Encoding) -> Vec<u8> {
    // Step 1
    let mut result = vec![];

    // Step 2
    let charset = encoding.name();

    // Step 3
    for entry in form_data.iter_mut() {
        // 3.1
        if entry.name == "_charset_" && entry.ty == "hidden" {
            entry.value = FormDatumValue::String(DOMString::from(charset.clone()));
        }
        // TODO: 3.2

        // Step 4
        // https://tools.ietf.org/html/rfc7578#section-4
        // NOTE(izgzhen): The encoding here expected by most servers seems different from
        // what spec says (that it should start with a '\r\n').
        let mut boundary_bytes = format!("--{}\r\n", boundary).into_bytes();
        result.append(&mut boundary_bytes);
        let mut content_disposition = ContentDisposition {
            disposition: DispositionType::Ext("form-data".to_owned()),
            parameters: vec![DispositionParam::Ext("name".to_owned(), String::from(entry.name.clone()))]
        };

        match entry.value {
            FormDatumValue::String(ref s) => {
                let mut bytes = format!("Content-Disposition: {}\r\n\r\n{}",
                                        content_disposition, s).into_bytes();
                result.append(&mut bytes);
            }
            FormDatumValue::File(ref f) => {
                content_disposition.parameters.push(
                    DispositionParam::Filename(Charset::Ext(String::from(charset.clone())),
                                               None,
                                               f.name().clone().into()));
                // https://tools.ietf.org/html/rfc7578#section-4.4
                let content_type = ContentType(f.upcast::<Blob<TH>>().Type()
                                                .parse().unwrap_or(mime!(Text / Plain)));
                let mut type_bytes = format!("Content-Disposition: {}\r\ncontent-type: {}\r\n\r\n",
                                             content_disposition,
                                             content_type).into_bytes();
                result.append(&mut type_bytes);

                let mut bytes = f.upcast::<Blob<TH>>().get_bytes().unwrap_or(vec![]);

                result.append(&mut bytes);
            }
        }
    }

    let mut boundary_bytes = format!("\r\n--{}--", boundary).into_bytes();
    result.append(&mut boundary_bytes);

    result
}

// https://tools.ietf.org/html/rfc7578#section-4.1
pub fn generate_boundary() -> String {
    let i1 = random::<u32>();
    let i2 = random::<u32>();

    format!("---------------------------{0}{1}", i1, i2)
}
