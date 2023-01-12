#![windows_subsystem = "windows"]

use serde_derive::Deserialize;
use tray_item::TrayItem;

extern crate clipboard;
use clipboard::{ClipboardContext, ClipboardProvider};
use enigo::{Enigo, Key, KeyboardControllable};

extern crate serialport;
use serialport::prelude::*;

use std::{fs, io, time::Duration};

extern crate winrt_notification;
use winrt_notification::{Sound, Toast};

#[derive(Deserialize)]
struct Config {
    port: String,
    rate: u32,
}
struct Card {
    rfid: String,
    id: u32,
}

fn main() {
    let mut tray = TrayItem::new("CardGetter", "start_icon").unwrap();

    let config_file =
        fs::read_to_string("data/config.toml").expect("Something went wrong reading the file");
    let config: Config = toml::from_str(&config_file).unwrap();

    tray.add_label(format!("{}:{}", &config.port, &config.rate).as_str())
        .unwrap();

    tray.add_menu_item("Quit", move || {
        println!("Quit");
        std::process::exit(1);
    })
    .unwrap();

    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(800);
    settings.baud_rate = config.rate;
    settings.stop_bits = StopBits::One;

    match serialport::open_with_settings(&config.port, &settings) {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 30];
            println!(
                "Receiving data on {} at {} baud:",
                &config.port, &config.rate
            );
            loop {
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(_) => {
                        let card_info = String::from_utf8_lossy(&serial_buf[..]);
                        if !card_info.contains("No card") {
                            //card_info[0] -- Название..
                            //card_info[1] -- HEX-код с карты..
                            let card_info: Vec<&str> =
                                card_info.split(|c| c == '[' || c == ']').collect();
                            //println!("{:?}", card_info); //Информация с порта
                            match card_info[0] {
                                "Mifare" => {
                                    //Преобразовываю HEX-строку в десятичное число.
                                    match u32::from_str_radix(card_info[1], 16) {
                                        Ok(id) => {
                                            let card = Card {
                                                rfid: String::from(card_info[0]),
                                                id,
                                            };
                                            println!("Тип карты: {}.\n{}", card.rfid, card.id);
                                            println!("{ }", "--------------");
                                            //Копирую номер в буфер-обмена
                                            let mut ctx: ClipboardContext =
                                                ClipboardProvider::new().unwrap();
                                            ctx.set_contents(card.id.to_owned().to_string())
                                                .unwrap();

                                            //ctrl + v emulate
                                            let mut enigo = Enigo::new();
                                            enigo.key_down(Key::Control);
                                            enigo.key_click(Key::Raw(86));
                                            enigo.key_up(Key::Control);
                                            enigo.key_click(Key::Raw(13));
                                        }
                                        Err(e) => println!("{}", e),
                                    }
                                }
                                "Em-Marine" => {
                                    println!("Em-Marine");
                                    println!("{}", "--------------");
                                    let mut ctx: ClipboardContext =
                                        ClipboardProvider::new().unwrap();
                                    ctx.set_contents("-Em-Marine-".to_string()).unwrap();
                                    Toast::new(Toast::POWERSHELL_APP_ID)
                                        .title("Em-Marine")
                                        .sound(Some(Sound::SMS))
                                        .duration(winrt_notification::Duration::Short)
                                        .show()
                                        .expect("unable to toast");
                                }
                                _ => {
                                    println!("Карта не распознана");
                                    println!("{}", "--------------");
                                    let mut ctx: ClipboardContext =
                                        ClipboardProvider::new().unwrap();
                                    ctx.set_contents("--------".to_string()).unwrap();
                                    Toast::new(Toast::POWERSHELL_APP_ID)
                                        .title("Карта не распознана")
                                        .text1("(╯°□°）╯︵ ┻━┻")
                                        .sound(Some(Sound::SMS))
                                        .duration(winrt_notification::Duration::Short)
                                        .show()
                                        .expect("unable to toast");
                                }
                            }
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", &config.port, e);
            println!(
                "Подключите считыватель\nВыключите программы использующие {}",
                &config.port
            );
            Toast::new(Toast::POWERSHELL_APP_ID)
                .title("Проверьте COM-port")
                .text1(
                    format!(
                        "Подключите считыватель\nВыключите программы использующие {}",
                        &config.port
                    )
                    .as_str(),
                )
                .sound(Some(Sound::SMS))
                .duration(winrt_notification::Duration::Short)
                .show()
                .expect("unable to toast");
            ::std::process::exit(1);
        }
    }
}
