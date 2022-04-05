use anyhow::Result;
use seed::{prelude::*, *};
use std::io::Read;

const FRAME_WIDTH: usize = 2000;
const FRAME_HEIGHT: usize = 2000;

struct Frame(Vec<u8>);

struct Model {
    active: bool,
    frames: Vec<Frame>,
    current_frame: Option<usize>,
}

enum Msg {
    Start,
    Stop,
    Forward,
    Backward,
    OnTick,
    DataFetched(fetch::Result<Vec<u8>>),
}

impl Frame {
    fn get_offset(x: usize, y: usize) -> usize {
        x * FRAME_WIDTH + y
    }

    fn get_pixel(&self, x: usize, y: usize) -> (u8, u8, u8, u8) {
        let i = Self::get_offset(x, y);
        let r = self.0[i];
        let g = self.0[i + 1];
        let b = self.0[i + 2];
        let a = self.0[i + 3];
        (r, g, b, a)
    }

    fn view_pixel(&self, x: usize, y: usize) -> Node<Msg> {
        let (r, g, b, a) = self.get_pixel(x, y);
        div![style![St::BackgroundColor => format!("rgba({r}, {g}, {b}, {a})")],]
    }

    fn view(&self) -> Vec<Node<Msg>> {
        (0..FRAME_HEIGHT)
            .into_iter()
            .map(|y| {
                let pixels =
                    (0..FRAME_WIDTH).into_iter().map(|x| self.view_pixel(x, y));
                div![C!["timelapse-row"], pixels]
            })
            .collect()
    }
}

impl Model {
    fn import_data(&mut self, data: Vec<u8>) -> Result<()> {
        let mut pixels = Vec::new();
        let mut gz = flate2::read::GzDecoder::new(&*data);
        gz.read_to_end(&mut pixels)?;

        // Number of pixels times the size it takes to encode one, i.e. 4 u8
        let frame_size = FRAME_WIDTH * FRAME_HEIGHT * 4;

        let mut i = 1;
        self.frames = pixels.chunks(frame_size).filter_map(|pixels| {
            log::info!("processing frame {i}");
            i += 1;
            if pixels.len() != frame_size {
                return None
            }
            Some(Frame(pixels.iter().cloned().collect()))
        }).collect();

        if self.frames.len() != 0 {
            self.current_frame = Some(0);
        }

        Ok(())
    }

    fn forward(&mut self) {
        if self.current_frame.is_none() {
            return;
        }

        if self.current_frame.unwrap() != self.frames.len() - 1 {
            self.current_frame = Some(self.current_frame.unwrap() + 1);
        }
    }

    fn backward(&mut self) {
        if self.current_frame.is_none() {
            return;
        }

        if self.current_frame.unwrap() != 0 {
            self.current_frame = Some(self.current_frame.unwrap() - 1);
        }
    }

    fn view(&self) -> Node<Msg> {
        if self.current_frame.is_none() {
            return empty![];
        }

        let frame = &self.frames[self.current_frame.unwrap()];

        div![C!["timelapse"], frame.view(),]
    }
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    log::info!("In init");
    orders
        .stream(streams::interval(1000, || Msg::OnTick))
        .perform_cmd(async {
            Msg::DataFetched(
                async {
                    fetch("/data.bin.gz")
                        .await?
                        .check_status()?
                        .bytes()
                        .await
                }
                .await,
            )
        });

    Model {
        active: false,
        frames: Vec::new(),
        current_frame: None,
    }
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::Start => model.active = true,
        Msg::Stop => model.active = false,
        Msg::Forward => model.forward(),
        Msg::Backward => model.backward(),
        Msg::OnTick => {
            if model.active {
                model.forward()
            }
        }
        Msg::DataFetched(Ok(data)) => match model.import_data(data) {
            Err(error) => error!("Error decoding data", error),
            _ => (),
        },
        Msg::DataFetched(Err(error)) => error!("Error fetching data", error),
    }
}

fn view(model: &Model) -> Node<Msg> {
    div![
        section![
            C!["timelapse-container"],
            header![
                C!["timelapse-header"],
                h1!["An always updating timelapse of /r/place"],
            ],
            section![
                C!["timelapse-area"],
                model.view(),
                div![
                    C!["timelapse-buttons"],
                    button!["Start", ev(Ev::Click, |_| Msg::Start),],
                    button!["Stop", ev(Ev::Click, |_| Msg::Stop),],
                    button!["Forward", ev(Ev::Click, |_| Msg::Forward),],
                    button!["Backward", ev(Ev::Click, |_| Msg::Backward),],
                ],
            ],
        ],
        footer![
            C!["timelapse-footer"],
            strong![
                C!["timelapse-footer-text"],
                a![
                    attrs! {
                        At::Href => "/about.html",
                    },
                    "About",
                ],
            ]
        ],
    ]
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    wasm_logger::init(wasm_logger::Config::default());
    log::warn!("Starting seed app");
    App::start("app", init, update, view);
}
