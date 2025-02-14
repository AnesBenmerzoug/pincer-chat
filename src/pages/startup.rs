use futures::FutureExt;
use gtk::prelude::*;
use relm4::prelude::*;
use std::{thread, time};
use tracing;

#[derive(Debug)]
pub struct StartUpPage {}

#[derive(Debug)]
pub enum StartUpPageInputMsg {
    CheckIfReady,
}

#[derive(Debug)]
pub enum StartUpPageOutputMsg {
    Ready,
}

#[derive(Debug)]
pub enum StartUpPageCmdMsg {
    Ready,
}

#[relm4::component(pub)]
impl Component for StartUpPage {
    type Init = ();
    type Input = StartUpPageInputMsg;
    type Output = StartUpPageOutputMsg;
    type CommandOutput = StartUpPageCmdMsg;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_vexpand: true,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,

            gtk::Spinner {
                set_spinning: true,
            },
            gtk::Label {
                set_label: "Starting up application...",
            },
        },
    }

    fn init(_: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = StartUpPage {};
        let widgets = view_output!();

        sender
            .input_sender()
            .emit(StartUpPageInputMsg::CheckIfReady);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            StartUpPageInputMsg::CheckIfReady => {
                sender.command(
                    |out: relm4::Sender<StartUpPageCmdMsg>, shutdown: relm4::ShutdownReceiver| {
                        shutdown
                            .register(async move {
                                let sleep_time = time::Duration::from_secs(1);
                                thread::sleep(sleep_time);
                                out.emit(StartUpPageCmdMsg::Ready);
                            })
                            // Perform task until a shutdown interrupts it
                            .drop_on_shutdown()
                            // Wrap into a `Pin<Box<Future>>` for return
                            .boxed()
                    },
                )
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            StartUpPageCmdMsg::Ready => {
                sender.output_sender().emit(StartUpPageOutputMsg::Ready);
            }
        }
    }
}
