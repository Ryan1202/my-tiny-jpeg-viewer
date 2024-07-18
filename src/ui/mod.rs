use iced::widget::{column, image, Button, Image};
use iced::{Command, Element};
use rfd::FileDialog;

use crate::get_jpeg_image;

pub struct App {
    pixels: image::Handle,
    width: u16,
    height: u16,
    img_path: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenFile,
    FileDecoded((usize, usize, Vec<u8>)),
}

impl iced::Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = iced::Theme;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            App {
                pixels: image::Handle::from_pixels(0, 0, vec![]),
                width: 0,
                height: 0,
                img_path: String::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("JpegViewer")
    }

    fn view(&self) -> Element<Self::Message> {
        column![
            Button::new("Open File").on_press(Message::OpenFile),
            Image::new(self.pixels.clone())
                .width(self.width)
                .height(self.height)
        ]
        .into()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::OpenFile => {
                if let Some(res) = FileDialog::new()
                    .set_title("Open File")
                    .add_filter("jpeg", &["jpeg", "jpg"])
                    .pick_file()
                {
                    self.img_path = res.display().to_string();
                }
                Command::perform(get_jpeg_image(self.img_path.clone()), Message::FileDecoded)
            }
            Message::FileDecoded((width, height, pixbuf)) => {
                self.pixels = image::Handle::from_pixels(width as u32, height as u32, pixbuf);
                self.width = width as u16;
                self.height = height as u16;
                Command::none()
            }
        }
    }
}
