use adw::prelude::*;
use clap::Parser;
use futures::FutureExt;
use notify_rust::Notification;
use relm4::{adw, gtk, Component, ComponentParts, ComponentSender, RelmApp, WidgetPlus};
use std::fs::read_to_string;
use std::path::PathBuf;

const MINUTE_LENGTH: f32 = 60.0;
const DELAY: f32 = 1.0;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    file: PathBuf,
    #[clap(short, long, value_parser, default_value_t = 1.0)]
    delay: f32,
}

#[derive(Debug)]
struct AppModel {
    elapsed_time: f32,
    wpm: f32,
    words: usize,
    started: bool,
    break_bool: bool,
    break_remaining_time: f32,

    file: PathBuf,

    pomodoro_duration: i32,
    short_break: i32,
    long_break: i32,
    long_break_streak: i32,

    first_count: usize,
}

#[derive(Debug)]
enum AppMsg {
    Start,
}

#[derive(Debug)]
enum CmdOut {
    Tick,
}

#[relm4::component]
impl Component for AppModel {
    /// The type of data that this component will be initialized with.
    type Init = AppModel;
    /// The type of the messages that this component can receive.
    type Input = AppMsg;
    /// The type of the messages that this component can send.
    type Output = ();
    /// A data structure that contains the widgets that you will need to update.
    type Widgets = AppWidgets;
    /// Messages which are received from commands executing in the background.
    type CommandOutput = CmdOut;

    view! {
        #[root]
        gtk::Window {
            set_title: Some("Simple app"),
            set_default_width: 300,
            set_default_height: 100,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::Label {
                    #[watch]
                    set_label: &format!(
                        "time {}:{:02}",
                        model.elapsed_time.floor(),
                        ((model.elapsed_time - model.elapsed_time.floor()) * MINUTE_LENGTH) as i32
                    ),
                    set_margin_all: 5,
                    #[watch]
                    set_visible: !model.break_bool,
                },

                gtk::Label {
                    #[watch]
                    set_label: &format!(
                        "time {}:{:02}",
                        model.break_remaining_time.floor(),
                        ((model.break_remaining_time - model.break_remaining_time.floor()) * MINUTE_LENGTH) as i32
                    ),
                    set_margin_all: 5,
                    #[watch]
                    set_visible: model.break_bool,
                },

                gtk::Label {
                    #[watch]
                    set_label: &format!("wpm: {:.2}", model.wpm),
                    set_margin_all: 5,
                },

                gtk::Label {
                    #[watch]
                    set_label: &format!("words: {}", model.words),
                    set_margin_all: 5,
                },

                gtk::Button {
                    set_label: "Start",
                    #[watch]
                    set_visible: !model.started,
                    connect_clicked[sender] => move |_| {
                        sender.input(AppMsg::Start);
                    },
                },
            },
        }
    }

    /// Initialize the UI and model.
    fn init(
        init: Self::Init,
        window: &Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = init;

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: relm4::ComponentSender<Self>) {
        match message {
            AppMsg::Start => {
                self.started = true;
                sender.command(|out, shutdown| {
                    shutdown
                        .register(async move {
                            loop {
                                out.send(CmdOut::Tick);
                                tokio::time::sleep(std::time::Duration::from_secs_f32(DELAY)).await;
                            }
                        })
                        .drop_on_shutdown()
                        .boxed()
                });
            }
        }
    }

    fn update_cmd(&mut self, message: Self::CommandOutput, _sender: ComponentSender<Self>) {
        match message {
            CmdOut::Tick => {
                if self.break_remaining_time <= 0.0 {
                    self.elapsed_time += DELAY / MINUTE_LENGTH;
                    self.words = count(&self.file) - self.first_count;
                    self.wpm = self.words as f32 / self.elapsed_time;

                    if self.elapsed_time as i32 != 0
                        && self.elapsed_time as i32 % self.pomodoro_duration == 0
                        && (self.elapsed_time - self.elapsed_time.floor()) * MINUTE_LENGTH >= 0.00
                        && (self.elapsed_time - self.elapsed_time.floor()) * MINUTE_LENGTH < DELAY
                    {
                        if self.elapsed_time as i32
                            % (self.pomodoro_duration * self.long_break_streak)
                            == 0
                        {
                            Notification::new()
                                .summary("Nice work, it's been a while, take a break man")
                                .show()
                                .unwrap();
                            self.break_remaining_time = self.long_break as f32;
                        } else {
                            Notification::new()
                                .summary("Nice work, shake your legs! take a drink!")
                                .show()
                                .unwrap();
                            self.break_remaining_time = self.short_break as f32;
                        }
                        self.break_bool = true;
                    }
                } else {
                    self.break_remaining_time -= DELAY / MINUTE_LENGTH;
                    if self.break_remaining_time <= 0.0 {
                        self.break_bool = false;
                    }
                }
            }
        }
    }
}

fn main() {
    let args = Args::parse();

    let app: RelmApp = RelmApp::new("relm4.test.wpm_watcher");
    app.run::<AppModel>(AppModel {
        first_count: count(&args.file),
        file: args.file,
        wpm: 0 as f32,
        words: 0,
        started: false,
        long_break_streak: 4,
        pomodoro_duration: 15,
        long_break: 20,
        short_break: 10,
        elapsed_time: 0 as f32,
        break_bool: false,
        break_remaining_time: 0 as f32,
    });
}

fn count(file: &PathBuf) -> usize {
    return words_count::count(read_to_string(file).expect("Failed to open given file")).words;
}
