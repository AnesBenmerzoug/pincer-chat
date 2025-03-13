use std::cmp::Ordering;

use chrono::NaiveDateTime;
use gtk::prelude::*;
use relm4::prelude::*;
use relm4::typed_view::list::{RelmListItem, TypedListView};

use crate::assistant::database::models::Thread;

#[derive(Debug)]
pub struct ThreadListContainerComponent {
    list_view_wrapper: TypedListView<ThreadListItem, gtk::SingleSelection>,
}

#[derive(Debug)]
pub enum ThreadListContainerInputMsg {
    SelectThread(u32),
    CreateNewThread,
    AddThread(Thread),
}

#[derive(Debug)]
pub enum ThreadListContainerOutputMsg {
    CreateNewThread,
    GetThreadMessages(i64),
}

#[relm4::component(async, pub)]
impl AsyncComponent for ThreadListContainerComponent {
    type Init = Vec<Thread>;
    type Input = ThreadListContainerInputMsg;
    type Output = ThreadListContainerOutputMsg;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 5,
            set_spacing: 5,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_margin_all: 5,
                set_spacing: 5,

                gtk::SearchEntry {
                    set_hexpand: true,
                },

                gtk::Button {
                    set_icon_name: "list-add-symbolic",
                    connect_clicked => ThreadListContainerInputMsg::CreateNewThread,
                }
            },

            #[name = "scrolled_window"]
            gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                set_hexpand: true,
                set_vexpand: true,
                set_valign: gtk::Align::Fill,

                #[local_ref]
                thread_list -> gtk::ListView {
                    set_margin_all: 5,

                    connect_activate[sender] => move |_, position| {
                        sender.input(ThreadListContainerInputMsg::SelectThread(position))
                    },
                },
            },
        },
    }

    async fn init(
        threads: Self::Init,
        _root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let list_view_wrapper: TypedListView<ThreadListItem, gtk::SingleSelection> =
            TypedListView::new();

        let _ = threads
            .into_iter()
            .map(|thread| {
                list_view_wrapper
                    .insert_sorted(ThreadListItem::new(thread), ThreadListItem::reverse_cmp)
            })
            .collect::<Vec<_>>();

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
            ThreadListContainerInputMsg::CreateNewThread => {
                sender
                    .output_sender()
                    .emit(ThreadListContainerOutputMsg::CreateNewThread);
            }
            ThreadListContainerInputMsg::SelectThread(position) => {
                let thread_list_item = self
                    .list_view_wrapper
                    .get(position)
                    .expect("Getting thread item at position should work");
                let thread_id = thread_list_item.borrow().thread_id;
                sender
                    .output_sender()
                    .emit(ThreadListContainerOutputMsg::GetThreadMessages(thread_id));
            }
            /*
            ThreadListContainerInputMsg::DeleteThread(thread_id) => {
                sender
                    .output_sender()
                    .emit(ThreadListContainerOutputMsg::DeleteThread(thread_id));
                self.list_view_wrapper.remove(position);
            }*/
            ThreadListContainerInputMsg::AddThread(thread) => {
                self.list_view_wrapper
                    .insert_sorted(ThreadListItem::new(thread), ThreadListItem::reverse_cmp);
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ThreadListItem {
    thread_id: i64,
    title: String,
    last_updated_at: NaiveDateTime,
}

impl ThreadListItem {
    fn new(thread: Thread) -> Self {
        Self {
            thread_id: thread.id,
            title: thread.title,
            last_updated_at: thread.last_updated_at,
        }
    }

    fn reverse_cmp(&self, other: &Self) -> Ordering {
        other.cmp(&self)
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
