use std::{ops::Deref, sync::Arc};

use btc_heritage_wallet::DatabaseItem;
use dioxus::prelude::*;

pub mod heir_list;
pub mod inheritances;
pub mod wallet;
pub mod wallet_list;

#[component]
fn TitledView(title: String, subtitle: String, children: Element) -> Element {
    rsx! {
        h1 { class: "text-6xl font-black text-center", "{title}" }
        h2 { class: "text-base font-light text-center", "{subtitle}" }
        div { class: "mb-4 h-px border-t border-solid border-gray-500" }
        { children }
    }
}

#[derive(Debug)]
struct DbItemWrapper<T: DatabaseItem>(Arc<T>);

impl<T: DatabaseItem> Clone for DbItemWrapper<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
impl<T: DatabaseItem> PartialEq for DbItemWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.name() == other.0.name()
    }
}
impl<T: DatabaseItem> From<T> for DbItemWrapper<T> {
    fn from(value: T) -> Self {
        Self(Arc::new(value))
    }
}
impl<T: DatabaseItem> Deref for DbItemWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
