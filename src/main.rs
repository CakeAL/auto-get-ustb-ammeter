use app::App;
use iced::Font;

mod app;
mod request;

#[tokio::main]
async fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .font(include_bytes!("../AlibabaPuHuiTi-3-55-Regular.ttf"))
        .default_font(Font::with_name("Alibaba PuHuiTi 3.0"))
        .run_with(App::init)
}
