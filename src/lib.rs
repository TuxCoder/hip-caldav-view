use chrono::{prelude::*, LocalResult};
use chrono_tz::Tz;
use gloo_net::http::Request;
use std::{default, time::SystemTime};
use yew::{prelude::*, virtual_dom::VNode};
//use serde::Deserialize;
/*use yew::{
    format::{Json, Nothing},
    prelude::*,
    services::fetch::{FetchService, FetchTask, Request, Response},
};*/

#[derive(Debug, Default)]
pub struct CalendarEvent {
    pub name: String,
    pub desciption: String,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
}

#[derive(Debug, Default)]
pub struct CalendarData {
    pub events: Vec<CalendarEvent>,
}

#[derive(Debug, Default)]
pub struct CaldavViewer {
    calendar: Option<CalendarData>,
}

pub enum Msg {
    Loaded,
    SetCalendarData(CalendarData),
}

fn parse_calendar(response: String) -> CalendarData {

    let reader = ical::IcalParser::new(response.as_bytes());
    let mut data = CalendarData::default();

    for line in reader {
        match line {
            Ok(ical_data) => {
                //log::info!("data: {:?}", data.events);
                for event in ical_data.events {
                    let mut start: Option<DateTime<Local>> = None;
                    let mut end: Option<DateTime<Local>> = None;
                    let mut name = None;
                    let mut desciption = None;
                    for prop in event.properties {
                        if prop.name == "SUMMARY" {
                            name = Some(prop.value.unwrap_or_default());
                        } else if prop.name == "DESCRIPTION" {
                            desciption = Some(prop.value.unwrap_or_default());
                        } else if prop.name == "DTSTART" {
                            let time = prop.value.unwrap_or_default();

                            let tz = chrono_tz::Europe::Berlin;

                            match chrono::NaiveDateTime::parse_from_str(
                                time.as_ref(),
                                "%Y%m%dT%H%M%S",
                            ) {
                                Ok(time) => {
                                    match tz.from_local_datetime(&time) {
                                        LocalResult::Single(time) => {
                                            //let utc = NaiveDate::from_
                                            start = Some(time.with_timezone(&Local));
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

                            let mut tz = chrono_tz::Europe::Berlin;
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
                            /*let tz_str = prop.params.into_iter().filter(|v|
                                false
                            ).collect();*/

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
                        }
                    }
                    if name.is_some() && start.is_some() && end.is_some() {
                        //add successfully parsed data
                        data.events.push(CalendarEvent {
                            name: name.unwrap(),
                            start: start.unwrap(),
                            end: end.unwrap(),
                            desciption: desciption.unwrap_or_else(|| "".to_string()),
                        })
                    }
                }
            }
            Err(_) => {
                log::error!("in parsing");
            }
        }
    }
    data
}


impl Component for CaldavViewer {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async {
            let response: String =
                Request::get("/hip.calendar/1e9b83e9-ad64-c8ec-89b9-6c79fcbe2742/")
                    .send()
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap();

            //let buf = BufReader::new(data);
            let data = parse_calendar(response);


            Msg::SetCalendarData(data)
        });

        Self { calendar: None }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetCalendarData(data) => self.calendar = Some(data),
            Msg::Loaded => {}
        }

        true
    }

    fn view(&self, _: &Context<Self>) -> Html {
        let eventlist: Option<VNode> = self.calendar.as_ref().map(|calendar_event| {
            calendar_event.events.iter().map(|event| 
                html! {
                    <div >
                    <h2> {event.name.clone() } </h2>
                    <p>
                        { format!("StartTime:{:}",  event.start) }
                    </p>
                    </div>
                }
            ).collect()
        });

        html! {
        <div>
            <h1>{ "Hip Public events" }</h1>
            { for eventlist }
        </div>
        }
    }
}
