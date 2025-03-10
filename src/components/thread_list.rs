use std::cmp::Ordering;

use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use gtk::prelude::*;
use relm4::prelude::*;
use relm4::typed_view::list::{RelmListItem, TypedListView};

use crate::assistant::database::models::Thread;

#[derive(Debug, PartialEq, Eq)]
struct ThreadListItem {
    title: String,
    last_updated_at: NaiveDateTime,
}

impl ThreadListItem {
    fn new(thread: Thread) -> Self {
        Self {
            title: thread.title,
            last_updated_at: thread.last_updated_at,
        }
    }
}

impl PartialOrd for ThreadListItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.last_updated_at.cmp(&other.last_updated_at))
    }
}

impl Ord for ThreadListItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.last_updated_at.cmp(&other.last_updated_at)
    }
}

struct ThreadListItemWidgets {
    title: gtk::Label,
    timestamp: gtk::Label,
}

impl RelmListItem for ThreadListItem {
    type Root = gtk::Box;
    type Widgets = ThreadListItemWidgets;

    fn setup(_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        relm4::view! {
            root = gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                #[name = "title"]
                gtk::Label,

                #[name = "timestamp"]
                gtk::Label,
            }
        }

        let widgets = Self::Widgets { title, timestamp };

        (root, widgets)
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _: &mut Self::Root) {
        let Self::Widgets { title, timestamp } = widgets;

        title.set_label(&*self.title);
        timestamp.set_label(&*self.last_updated_at.format("%d %B %Y at %R").to_string());
    }
}

#[derive(Debug)]
pub struct ThreadListContainerComponent {
    list_view_wrapper: TypedListView<ThreadListItem, gtk::SingleSelection>,
}

#[derive(Debug)]
pub enum ThreadListContainerInputMsg {
    AddThread(Thread),
}

#[relm4::component(async, pub)]
impl AsyncComponent for ThreadListContainerComponent {
    type Init = ();
    type Input = ThreadListContainerInputMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[name = "scrolled_window"]
        gtk::ScrolledWindow {
            set_hscrollbar_policy: gtk::PolicyType::Never,
            set_hexpand: true,

            #[local_ref]
            thread_list -> gtk::ListView {},
        },
    }

    async fn init(
        _: Self::Init,
        _root: Self::Root,
        _sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let list_view_wrapper: TypedListView<ThreadListItem, gtk::SingleSelection> =
            TypedListView::with_sorting();

        let model = ThreadListContainerComponent { list_view_wrapper };

        let thread_list = &model.list_view_wrapper.view;

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _: &Self::Root,
    ) {
        match message {
            ThreadListContainerInputMsg::AddThread(thread) => {
                self.list_view_wrapper.append(ThreadListItem::new(thread));
            }
        }
    }
}
