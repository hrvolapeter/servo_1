/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::HTMLMetaElementBinding;
use dom::bindings::codegen::Bindings::HTMLMetaElementBinding::HTMLMetaElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::{DomRoot, MutNullableDom, RootedReference};
use dom::bindings::str::DOMString;
use dom::cssstylesheet::CSSStyleSheet;
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::htmlelement::HTMLElement;
use dom::htmlheadelement::HTMLHeadElement;
use dom::node::{Node, UnbindContext, document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use parking_lot::RwLock;
use servo_arc::Arc;
use servo_config::prefs::PREFS;
use std::sync::atomic::AtomicBool;
use style::attr::AttrValue;
use style::media_queries::MediaList;
use style::str::HTML_SPACE_CHARACTERS;
use style::stylesheets::{Stylesheet, StylesheetContents, CssRule, CssRules, Origin, ViewportRule};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLMetaElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement,
    #[ignore_malloc_size_of = "Arc"]
    stylesheet: DomRefCell<Option<Arc<Stylesheet>>>,
    cssom_stylesheet: MutNullableDom<CSSStyleSheet>,
}

impl<TH: TypeHolderTrait> HTMLMetaElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>) -> HTMLMetaElement<TH> {
        HTMLMetaElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            stylesheet: DomRefCell::new(None),
            cssom_stylesheet: MutNullableDom::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLMetaElement<TH>> {
        Node::reflect_node(Box::new(HTMLMetaElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLMetaElementBinding::Wrap)
    }

    pub fn get_stylesheet(&self) -> Option<Arc<Stylesheet>> {
        self.stylesheet.borrow().clone()
    }

    pub fn get_cssom_stylesheet(&self) -> Option<DomRoot<CSSStyleSheet>> {
        self.get_stylesheet().map(|sheet| {
            self.cssom_stylesheet.or_init(|| {
                CSSStyleSheet::new(&window_from_node(self),
                                   self.upcast::<Element<TH>>(),
                                   "text/css".into(),
                                   None, // todo handle location
                                   None, // todo handle title
                                   sheet)
            })
        })
    }

    fn process_attributes(&self) {
        let element = self.upcast::<Element<TH>>();
        if let Some(name) = element.get_attribute(&ns!(), &local_name!("name")).r() {
            let name = name.value().to_ascii_lowercase();
            let name = name.trim_matches(HTML_SPACE_CHARACTERS);

            if name == "viewport" {
                self.apply_viewport();
            }

            if name == "referrer" {
                self.apply_referrer();
            }
        }
    }

    fn apply_viewport(&self) {
        if !PREFS.get("layout.viewport.enabled").as_boolean().unwrap_or(false) {
            return;
        }
        let element = self.upcast::<Element<TH>>();
        if let Some(content) = element.get_attribute(&ns!(), &local_name!("content")).r() {
            let content = content.value();
            if !content.is_empty() {
                if let Some(translated_rule) = ViewportRule::from_meta(&**content) {
                    let document = document_from_node(self);
                    let shared_lock = document.style_shared_lock();
                    let rule = CssRule::Viewport(Arc::new(shared_lock.wrap(translated_rule)));
                    let sheet = Arc::new(Stylesheet {
                        contents: StylesheetContents {
                            rules: CssRules::new(vec![rule], shared_lock),
                            origin: Origin::Author,
                            namespaces: Default::default(),
                            quirks_mode: document.quirks_mode(),
                            url_data: RwLock::new(window_from_node(self).get_url()),
                            source_map_url: RwLock::new(None),
                            source_url: RwLock::new(None),
                        },
                        media: Arc::new(shared_lock.wrap(MediaList::empty())),
                        shared_lock: shared_lock.clone(),
                        disabled: AtomicBool::new(false),
                    });
                    *self.stylesheet.borrow_mut() = Some(sheet.clone());
                    document.add_stylesheet(self.upcast(), sheet);
                }
            }
        }
    }

    fn process_referrer_attribute(&self) {
        let element = self.upcast::<Element<TH>>();
        if let Some(name) = element.get_attribute(&ns!(), &local_name!("name")).r() {
            let name = name.value().to_ascii_lowercase();
            let name = name.trim_matches(HTML_SPACE_CHARACTERS);

            if name == "referrer" {
                self.apply_referrer();
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#meta-referrer>
    fn apply_referrer(&self) {
        if let Some(parent) = self.upcast::<Node<TH>>().GetParentElement() {
            if let Some(head) = parent.downcast::<HTMLHeadElement>() {
                head.set_document_referrer();
            }
        }
    }
}

impl<TH> HTMLMetaElementMethods for HTMLMetaElement<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-meta-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-meta-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-meta-content
    make_getter!(Content, "content");

    // https://html.spec.whatwg.org/multipage/#dom-meta-content
    make_setter!(SetContent, "content");
}

impl<TH> VirtualMethods for HTMLMetaElement<TH> {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        if tree_in_doc {
            self.process_attributes();
        }
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("name") => AttrValue::from_atomic(value.into()),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        if let Some(s) = self.super_type() {
            s.attribute_mutated(attr, mutation);
        }

        self.process_referrer_attribute();
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(context);
        }

        if context.tree_in_doc {
            self.process_referrer_attribute();

            if let Some(s) = self.stylesheet.borrow_mut().take() {
                document_from_node(self).remove_stylesheet(self.upcast(), &s);
            }
        }
    }
}
