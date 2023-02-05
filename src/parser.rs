use std::collections::HashMap;

use chrono_tz::Tz;
use chrono::{DateTime, LocalResult, Local, TimeZone, Utc};
use ical::parser::ParserError;

#[derive(Debug, Clone)]
pub struct CalendarEvent {
    pub name: String,
    pub desciption: String,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
}

#[derive(Debug, Default, Clone)]
pub struct CalendarData {
    pub events: Vec<CalendarEvent>,
}


#[derive(Debug)]
pub enum Error {
    ParseError(ParserError),
    TimeZone(),
}

pub fn parse_calendar(response: &str) -> Result<CalendarData, Error> {

    //let calendar = Calendar::new_from_data(response);

    let reader = ical::IcalParser::new(response.as_bytes());
    let mut data = CalendarData::default();

    for line in reader {
        match line {
            Ok(ical_data) => {
                //log::info!("data: {:?}", data.events);
                for event in ical_data.events {
                    let mut start: Option<DateTime<Local>> = None;
                    let mut start_tz: Option<Tz> = None;
                    let mut end: Option<DateTime<Local>> = None;
                    let mut name = None;
                    let mut desciption = None;
                    let mut rrule: HashMap<String, String> = HashMap::new();
                    for prop in event.properties {
                        if prop.name == "SUMMARY" {
                            name = Some(prop.value.unwrap_or_default());
                        } else if prop.name == "DESCRIPTION" {
                            desciption = Some(prop.value.unwrap_or_default());
                        } else if prop.name == "DTSTART" {
                            let time = prop.value.unwrap_or_default();

                            let mut tz = chrono_tz::Europe::Berlin; // default timezone
                            for (param_name, param_values) in prop.params.unwrap() {

                                if param_name == "TZID" {
                                    match param_values[0].parse() {
                                        Ok(timezone) => {
                                            tz = timezone;
                                        }
                                        Err(_) => {
                                            log::error!("tzid parse error");
                                            return Err(Error::TimeZone());
                                        }
                                    }
                                }
                            }

                            match chrono::NaiveDateTime::parse_from_str(
                                time.as_ref(),
                                "%Y%m%dT%H%M%S",
                            ) {
                                Ok(time) => {
                                    match tz.from_local_datetime(&time) {
                                        LocalResult::Single(time) => {
                                            start = Some(time.with_timezone(&Local));
                                            start_tz = Some(tz);
                                        }
                                        _ => {
                                            log::error!("time convert errror");
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::error!("{:?}", e)
                                }
                            }
                        } else if prop.name == "DTEND" {
                            let time = prop.value.unwrap_or_default();

                            let mut tz = chrono_tz::Europe::Berlin; // default timezone
                            for (param_name, param_values) in prop.params.unwrap() {

                                if param_name == "TZID" {
                                    match param_values[0].parse() {
                                        Ok(timezone) => {
                                            tz = timezone
                                        }
                                        Err(_) => {
                                            log::error!("tzid parse error")
                                        }
                                    }
                                }
                            }

                            match chrono::NaiveDateTime::parse_from_str(
                                time.as_ref(),
                                "%Y%m%dT%H%M%S",
                            ) {
                                Ok(time) => {
                                    match tz.from_local_datetime(&time) {
                                        LocalResult::Single(time) => {
                                            end = Some(
                                                time.with_timezone(&Local), //DateTime<Local>::default()
                                            )
                                        }
                                        _ => {
                                            log::error!("time convert errror");
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::error!("{:?}", e)
                                }
                            }
                        } else if prop.name == "RRULE" {
                            for param in prop.value.unwrap().split(";") {
                                let mut split = param.split("=");
                                let (key, value)  = match (split.next(), split.next())  {
                                    (Some(key), Some(value)) => (key, value),
                                    _ => {
                                        break;
                                    }
                                };
                                rrule.insert(key.to_string(), value.to_string());
                            }
                        }
                    }
                    if name.is_some() && start.is_some() && end.is_some() {
                        let name = name.unwrap();
                        let start = start.unwrap();
                        let end = end.unwrap();
                        let desciption = desciption.unwrap_or_else(|| "".to_string());
                        //add successfully parsed data
                        let event = CalendarEvent {
                            name: name.clone(),
                            start: start,
                            end: end,
                            desciption: desciption.clone(),
                        };
                        data.events.push(event);

                        if rrule.contains_key("FREQ") && rrule.contains_key("UNTIL") {
                            let freq = rrule.get("FREQ").unwrap().as_str();
                            let duration = end - start;
                            match chrono::NaiveDateTime::parse_from_str(
                                rrule.get("UNTIL").unwrap(),
                                "%Y%m%dT%H%M%SZ",
                            ) {
                                Ok(until_raw) => {
                                    let until = Utc.from_utc_datetime(&until_raw);


                                    let periode = match freq {
                                        "WEEKLY" => {
                                            chrono::Duration::weeks(1)
                                        },
                                        _ => {
                                            log::error!("not implemented freq\n");
                                            chrono::Duration::weeks(100)
                                        }
                                    };
                                    let mut time_event_tz = start.with_timezone(&start_tz.unwrap());
                                    
                                    time_event_tz += periode;
                                    while time_event_tz < until {

                                        let event = CalendarEvent {
                                            name: name.clone(),
                                            start: time_event_tz.with_timezone(&Local),
                                            end: (time_event_tz + duration).with_timezone(&Local),
                                            desciption: desciption.clone(),
                                        };
                                        data.events.push(event);

                                        time_event_tz += periode;
                                    }

                                },
                                Err(_) => {
                                    log::error!("could not parse UNTIL date in rrule");
                                }
                            }
                        }
                    }
                }
            }
            Err(error) => {
                log::error!("in parsing {error}");
                return Err(Error::ParseError(error));
            }
        }
    }
    data.events.sort_by(|a, b| a.start.cmp(&b.start) );
    Ok(data)
}
