use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{BufRead, Seek, SeekFrom, Write},
    str::FromStr,
};

use chrono::{Local, NaiveDate};
use iced::{
    widget::{button, column, rich_text, row, scrollable, span, text, text::Span, text_input},
    Element, Task,
};
use std::io::BufReader;

use crate::request::get_ammeter;

#[derive(Debug, Default)]
pub struct Data {
    pub date: NaiveDate,
    pub remain: i32,
    pub average: f64,
}

#[derive(Debug)]
pub struct App {
    pub file: File,
    pub ammeter_number: Option<u32>,
    pub data: Vec<Data>,
    pub info: String,
    pub input_value: String,
    pub duration: i64,
    pub today_date: NaiveDate,
}

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    ButtonClicked(String),
    RequestFinished(Result<Option<i32>, String>),
    WriteData,
}

impl App {
    pub fn init() -> (Self, Task<Message>) {
        let path = env::current_exe()
            .expect("failed to get current exe path.")
            .parent()
            .expect("failed to get parent directory.")
            .join("ammeter_data.csv");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(true)
            .open(&path)
            .expect("open file failed.");
        let metadata = fs::metadata(path).expect("read metadata failed.");
        // if 新建文件
        if metadata.len() == 0 {
            return (
                App {
                    file,
                    ammeter_number: None,
                    data: vec![],
                    info: String::new(),
                    input_value: String::new(),
                    duration: -1,
                    today_date: NaiveDate::from(Local::now().naive_local()),
                },
                Task::none(),
            );
        }
        let mut lines = BufReader::new(file.try_clone().unwrap()).lines();
        let mut data = vec![];
        let first_line = lines.next().unwrap().unwrap();
        let ammeter_number = first_line
            .split(',')
            .next()
            .unwrap()
            .trim()
            .parse::<u32>()
            .unwrap();
        for line in lines {
            let line = line.unwrap();
            let line = line.split(',').collect::<Vec<&str>>();
            let date = NaiveDate::from_str(line[1]).unwrap();
            let remain = line[2].parse::<i32>().unwrap();
            let average = line[3].parse::<f64>().unwrap();
            data.push(Data {
                date,
                remain,
                average,
            });
        }

        let app = App {
            file,
            ammeter_number: Some(ammeter_number),
            data,
            info: format!("当前电表号：{}，自动更新数据中···\n", ammeter_number),
            input_value: format!("{}", ammeter_number),
            duration: -1,
            today_date: NaiveDate::from(Local::now().naive_local()),
        };
        dbg!(&app);
        (
            app,
            Task::perform(
                async move { Message::ButtonClicked(format!("{}", ammeter_number)) },
                |m| m,
            ),
        )
    }

    pub fn title(&self) -> String {
        format!(
            "查询🐚电表余量：当前电表号：{}",
            self.ammeter_number.unwrap_or_default()
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InputChanged(value) => {
                self.input_value = value;
                Task::none()
            }
            Message::ButtonClicked(ammeter_number) => {
                dbg!(&ammeter_number);
                if let Some(data) = self.data.last() {
                    self.duration = self.today_date.signed_duration_since(data.date).num_days();
                    if self.duration < 1 {
                        self.info.push_str("距离上一次查询还没超过一天！\n");
                        return Task::none();
                    }
                }
                let ammeter_number = match ammeter_number.parse::<u32>() {
                    Ok(num) => num,
                    Err(e) => {
                        self.info.push_str(&format!("电表号应该是纯数字: {}\n", e));
                        0
                    }
                };
                self.ammeter_number = Some(ammeter_number);
                Task::perform(get_ammeter(ammeter_number), Message::RequestFinished)
            }
            Message::RequestFinished(res) => {
                dbg!(&res);
                match res {
                    Err(e) => self.info.push_str(&format!("{}\n", e)),
                    Ok(Some(kwh)) => {
                        if kwh < 30 {
                            self.info
                                .push_str("剩余电量小于 30 KWh，赶紧给学校打钱！\n");
                        }
                        let mut last_kwh = 0;
                        if let Some(data) = self.data.last() {
                            last_kwh = data.remain;
                        }
                        let average = (last_kwh - kwh) as f64 / self.duration as f64;
                        self.data.push(Data {
                            date: self.today_date,
                            remain: kwh,
                            average,
                        });
                        self.info
                            .push_str(&format!("查询电量成功！目前还剩余 {} Kwh\n", kwh));
                    }
                    Ok(None) => {
                        self.info.push_str(&format!(
                            "获取 No.{} 电表数据失败，请检查是否是一个正确的电表号！\n",
                            self.ammeter_number.unwrap_or_default()
                        ));
                    }
                }
                // dbg!(&self.data);
                Task::perform(async { Message::WriteData }, |msg| msg)
            }
            Message::WriteData => {
                self.file.seek(SeekFrom::Start(0)).unwrap();
                self.file.set_len(0).unwrap();
                if let Err(e) = write!(
                    self.file,
                    "{},Date,Remain(KWh),Average everyday usage since last date",
                    self.ammeter_number.unwrap_or_default()
                ) {
                    self.info.push_str(&format!("{}\n", e));
                }
                for line in self.data.iter() {
                    if let Err(e) = write!(
                        self.file,
                        "\n,{},{},{}",
                        line.date, line.remain, line.average
                    ) {
                        self.info.push_str(&format!("{}\n", e));
                    }
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let input = text_input("电表号", &self.input_value)
            .id("input-number")
            .on_input(Message::InputChanged);
        let button = button("点我查询").on_press(Message::ButtonClicked(self.input_value.clone()));
        let text = text(&self.info);

        let table_firstline = rich_text([span(
            "日期\t\t\t剩余(KWh)\t\t\t距离上一次查询平均每天使用度数\n",
        )]);
        let table_spans = self
            .data
            .iter()
            .map(|data| {
                span(format!(
                    "{}\t\t\t{}\t\t\t{}\n",
                    data.date, data.remain, data.average
                ))
            })
            .collect::<Vec<Span<'_, Message>>>();
        let table = rich_text(table_spans);

        let up = row![input, button].spacing(20);
        let down = row![
            scrollable(text),
            column![table_firstline, scrollable(table)].spacing(10)
        ]
        .spacing(30);
        let container = column![up, down].spacing(20);
        container.into()
    }
}
