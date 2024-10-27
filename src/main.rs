use std::env;
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::io::Write;
use std::str::FromStr;

use chrono::{Local, NaiveDate};

mod request;

#[tokio::main]
async fn main() {
    let mut path = env::current_exe().expect("failed to get current directory");
    path.pop();
    path.push("ammeter_data.csv");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)
        .expect("open file failed");
    let metadata = fs::metadata(path).expect("read metadata failed");
    let mut file_string = String::new();

    let ammeter_number = match metadata.len() {
        0 => {
            println!("Input the ammeter number: ");
            loop {
                let mut input_string = String::new();
                std::io::stdin()
                    .read_line(&mut input_string)
                    .expect("read your input failed");
                match input_string.trim().parse::<u32>() {
                    Err(_) => {
                        eprintln!("Not a right ammeter number 🤔! ")
                    }
                    Ok(num) => {
                        write!(
                            file,
                            "{num},Date,Remain(KWh),Average everyday usage since last date"
                        )
                        .expect("write to file failed");
                        break num;
                    }
                }
            }
        }
        _ => {
            file.read_to_string(&mut file_string)
                .expect("read file to string failed");
            match file_string.split(',').next().unwrap().trim().parse::<u32>() {
                Err(_) => {
                    eprintln!("the first line first data of ammeter_data.csv is not a right ammeter 🤔! Huh?");
                    return;
                }
                Ok(num) => num,
            }
        }
    };

    let today_date = NaiveDate::from(Local::now().naive_local());
    let last_data = file_string
        .split("\n")
        .last()
        .unwrap()
        .split(",")
        .collect::<Vec<&str>>();
    // dbg!(&last_data);
    let mut duration = -1;
    if file_string.len() > 0 {
        let last_date = NaiveDate::from_str(last_data[1]).unwrap_or_default();
        duration = today_date.signed_duration_since(last_date).num_days();
        if duration < 1 {
            eprintln!("It's been less than a day since the last ammeter data was updated!");
            return;
        }
    }
    match request::get_ammeter(ammeter_number).await {
        Ok(Some(kwh)) => {
            println!("The No.{} ammeter remains {} KWh.", ammeter_number, kwh);
            if kwh < 30 {
                eprintln!(
                    "The remaining is less than 30 KWh, hurry up sending money to the school~"
                );
            }
            if file_string.len() == 0 {
                write!(file, "\n,{},{},0", today_date.to_string(), kwh,)
                    .expect("write to file failed");
                return;
            };
            let last_kwh = last_data[2].parse::<i32>().unwrap_or(kwh);
            // dbg!(last_kwh, duration);
            let average_kwh = (last_kwh - kwh) as f64 / duration as f64;
            write!(
                file,
                "\n,{},{},{}",
                today_date.to_string(),
                kwh,
                average_kwh
            )
            .expect("write to file failed");
        }
        Ok(None) => eprintln!(
            "Get the No.{} ammeter data failed, check if it's a right ammeter number",
            ammeter_number
        ),
        Err(err) => {
            eprintln!("Get the No.{} ammeter data failed: {}", ammeter_number, err)
        }
    }
}
