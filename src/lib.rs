use chrono::Local;
use chrono_tz::Tz;
use gloo_net;
use gloo_net::http::Request;
use js_sys::{Array, Intl, Object, Reflect};
use wasm_bindgen::JsValue;
use yew::{prelude::*, virtual_dom::VNode};
use yew_hooks::{use_async, use_mount};
use thiserror;

mod parser;

pub fn get_client_timezone() -> Tz {
    let options = Intl::DateTimeFormat::new(&Array::new(), &Object::new()).resolved_options();

    let tz = Reflect::get(&options, &JsValue::from("timeZone"))
        .expect("Cannot get timeZone")
        .as_string()
        .expect("timeZone is not a String");

    tz.parse().unwrap()
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub url: String,
}

#[derive(Debug, thiserror::Error, Clone)]
enum Error {
    #[error("connection error")]
    Connection(),
    #[error("parsing error")]
    Parser(),
}

async fn load_data(url: &str) -> Result<String, Error> {
    let response: String = Request::get(url)
        .send()
        .await
        .or_else(|_| Err(Error::Connection()))?
        .text()
        .await
        .or_else(|_| Err(Error::Connection()))?;

    Ok(response)
}

#[function_component(CaldavViewer)]
pub fn caldav_viewer(props: &Props) -> Html {
    let calendar_data_result_async = {
        let props = props.clone();
        use_async(async move { load_data(&props.url).await })
    };

    {
        let calendar_data_result_async = calendar_data_result_async.clone();
        use_mount(move || {
            calendar_data_result_async.run();
            ()
        });
    }


    if let Some(calendar_data_result) = &calendar_data_result_async.data {
        let now = Local::now();
        let load_from = now - chrono::Duration::days(3);
        let soon = now + chrono::Duration::days(2);
        let local_timezone = get_client_timezone();

        let calendar_data = parser::parse_calendar(calendar_data_result).unwrap();
        let events = calendar_data.events;

        let eventlist: VNode =
            events.iter()
                .filter(|event| event.start > load_from)
                .map(|event| {
                    let status_class = if event.start < now { "old" } else if event.start < soon { "soon" } else {""};

                    html! {
                        <div class={status_class} >
                        <h2> {event.name.clone() } </h2>
                        <p>
                            { format!("StartTime:{}",  event.start.with_timezone(&local_timezone)) } <br />
                            { "Description:" } <br />
                            <p>
                                { for event.desciption.split("\\n").map(|line| html!{<> {line} <br /> </> }) }
                            </p>
                        </p>
                        </div>
                    }
                }).collect();

        return html! {
            <div>
                <h1>{ "Hip Public events" }</h1>
                <p>{ "CalDav URL:" }<a href={ props.url.clone() } >{ props.url.clone() }</a></p>
                { eventlist }
            </div>
        }
    }

    match &calendar_data_result_async.error {
        Some(_) => {
            return html! {
                <div>
                    {"Error!"}
                </div>
            };
        },
        None => {
            return html!{
                <div>
                    {"loading"}
                </div>
            }
        }
    }
}
/*
impl Component for CaldavViewer {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async {
            let response: String =
                Request::get(CALENDAR_URL)
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
        let now = Local::now();
        let load_from = now - chrono::Duration::days(3);
        let soon = now + chrono::Duration::days(2);
        let local_timezone = get_client_timezone();

        let eventlist: Option<VNode> = self.calendar.as_ref().map(|calendar_event| {
            calendar_event.events.iter()
                .filter(|event| event.start > load_from)
                .map(|event| {
                    let status_class = if event.start < now { "old" } else if event.start < soon { "soon" } else {""};

                    html! {
                        <div class={status_class} >
                        <h2> {event.name.clone() } </h2>
                        <p>
                            { format!("StartTime:{}",  event.start.with_timezone(&local_timezone)) } <br />
                            { "Description:" } <br />
                            <p>
                                { for event.desciption.split("\\n").map(|line| html!{<> {line} <br /> </> }) }
                            </p>
                        </p>
                        </div>
                    }
            }).collect()
        });

        html! {
        <div>
            <h1>{ "Hip Public events" }</h1>
            <p>{ "CalDav URL:" }<a href={ CALENDAR_URL } >{ CALENDAR_URL }</a></p>
            { for eventlist }
        </div>
        }
    }
}
*/
