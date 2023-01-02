use iced::alignment;
use iced::executor;
use iced::theme::{self, Theme};
use iced::time;
use iced::widget::{button, column, container, row, text};
use iced::{
    Alignment, Application, Command, Element, Length, Settings, Subscription,
};
use log::info;
use log::warn;
use serde::Deserialize;
use serde::Serialize;

use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize, Debug)]
pub struct SensorStatus {
    pub name: String,
    pub location: String,
    pub temperature: u8,
    pub high: u8,
    pub low: u8,
    pub humidity: u8
}

pub fn main() -> iced::Result {
    env_logger::init();
    Stopwatch::run(Settings::default())
}

struct Stopwatch {
    duration: Duration,
    state: State,
    data: Arc<Mutex<String>>
}

enum State {
    Idle,
    Ticking { last_tick: Instant },
}

#[derive(Debug, Clone)]
enum Message {
    Toggle,
    Reset,
    Tick(Instant),
    SayIt(String)
}

impl Application for Stopwatch {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Stopwatch, Command<Message>) {
        let sensor = String::from("");
        let arc_sensor = Arc::new(Mutex::new(sensor));
        let arc_sensor_clone = Arc::clone(&arc_sensor);
    
        thread::spawn(move || {
            let nc = nats::connect("10.5.1.216:4222").expect("connect failed");
            let sub = nc.subscribe("*.status").expect("");
    
            for msg in sub.messages() {
                info!("Received a request {msg:?}");
    
                let sensor_msg = msg.to_string();
                let sensor_status_string = String::from_utf8_lossy(&msg.data).to_string();
                let sensor_status: SensorStatus = serde_json::from_str(&sensor_status_string).unwrap();
    
                info!("{:?}", sensor_status);
    
                //let s = &arc_location_search_clone_3.lock().unwrap().to_owned();

                // if sensor_status.location.eq(&String::from("Office")) {
                //     info!("{:?}", sensor_status_string);
                //     *arc_sensor_clone_2.lock().unwrap() = sensor_status_string.clone();
                // }

                info!("{:?}", sensor_status_string);
                *arc_sensor_clone.lock().unwrap() = sensor_status_string.clone();
            }    
        });

        (
            Stopwatch {
                duration: Duration::default(),
                state: State::Idle,
                data: Arc::clone(&arc_sensor)
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Stopwatch - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        warn!("update!");
        match message {
            Message::SayIt(s) => {
                warn!("{}", s);
            },
            Message::Toggle => match self.state {
                State::Idle => {
                    self.state = State::Ticking {
                        last_tick: Instant::now(),
                    };
                }
                State::Ticking { .. } => {
                    self.state = State::Idle;
                }
            },
            Message::Tick(now) => {
                if let State::Ticking { last_tick } = &mut self.state {
                    self.duration += now - *last_tick;
                    *last_tick = now;
                }
            }
            Message::Reset => {
                self.duration = Duration::default();
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        info!("subscription");
        match self.state {
            State::Idle => Subscription::none(),
            State::Ticking { .. } => {
                time::every(Duration::from_millis(10)).map(Message::Tick)
            }
        }
    }

    fn view(&self) -> Element<Message> {
        const MINUTE: u64 = 60;
        const HOUR: u64 = 60 * MINUTE;

        let seconds = self.duration.as_secs();

        let sensor_data = text(format!("{}", self.data.lock().unwrap())).size(20);

        let duration = text(format!(
            "{:0>2}:{:0>2}:{:0>2}.{:0>2}",
            seconds / HOUR,
            (seconds % HOUR) / MINUTE,
            seconds % MINUTE,
            self.duration.subsec_millis() / 10,
        ))
        .size(40);

        let button = |label| {
            button(
                text(label).horizontal_alignment(alignment::Horizontal::Center),
            )
            .padding(10)
            .width(Length::Units(80))
        };

        let toggle_button = {
            let label = match self.state {
                State::Idle => "Start",
                State::Ticking { .. } => "Stop",
            };

            button(label).on_press(Message::Toggle)
        };

        let reset_button = button("Reset")
            .style(theme::Button::Destructive)
            .on_press(Message::Reset);

        //testing123
        let sne_button = {
            let label = match self.state {
                State::Idle => "->",
                State::Ticking { .. } => "-",
            };

            button(label).on_press(Message::SayIt(String::from("woohoo!")))
        };
        //

        let controls = row![toggle_button, reset_button, sne_button].spacing(20);

        let content = column![sensor_data, duration, controls]
            .align_items(Alignment::Center)
            .spacing(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}