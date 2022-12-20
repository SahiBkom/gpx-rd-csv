use quick_xml::events::attributes::{Attribute, Attributes};
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use rijksdriehoek::wgs84_to_rijksdriehoek;
use std::borrow::Borrow;
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::str::FromStr;

#[derive(Debug, Clone)]
struct Wpt {
    lat: f64,
    lng: f64,
    name: String,
}

pub fn get_attr_as_f64<N: AsRef<[u8]> + Sized>(bs: &BytesStart, attr_name: N) -> f64 {
    f64::from_str(
        (&String::from_utf8_lossy(
            bs.try_get_attribute(attr_name)
                .unwrap()
                .unwrap()
                .value
                .borrow(),
        ))
            .borrow(),
    )
    .unwrap()
}

fn main() {
    let xml = std::fs::read_to_string("/home/johan/proj/scouting/gpx-rd-csv/pw2023.gpx").unwrap();

    let mut reader = Reader::from_str(&xml);
    reader.trim_text(true);

    let mut count = 0;
    let mut txt = Vec::new();
    let mut buf = Vec::new();

    let mut wpt: Option<Wpt> = None;

    let mut wpts: Vec<Wpt> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            // exits the loop when reaching end of file
            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"wpt" => {
                    let lat = get_attr_as_f64(&e, b"lat");
                    let lng = get_attr_as_f64(&e, b"lon");

                    println!("attributes values: {:?} {:?}", lat, lng);
                    wpt = Some(Wpt {
                        lat,
                        lng,
                        name: "".to_string(),
                    })
                }

                b"name" => {
                    txt.clear();
                }

                _ => println!("TAG: {e:?}"),
            },
            Ok(Event::Text(e)) => txt.push(e.unescape().unwrap().into_owned()),

            Ok(Event::End(e)) => match e.name().as_ref() {
                b"name" => {
                    if let Some(mut wpt) = wpt.as_mut() {
                        wpt.name = txt.join("");
                        println!("{wpt:?}");
                        wpts.push(wpt.clone());
                    }
                }
                _ => (),
            },

            // There are several other `Event`s we do not consider here
            _ => (),
        }
        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    for wpt in wpts {
        let (x, y) = wgs84_to_rijksdriehoek(wpt.lat, wpt.lng);
        println!("\"{}\";{};{};{};{}", wpt.name, wpt.lat, wpt.lng, x, y);
    }
}
