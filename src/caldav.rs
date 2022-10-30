
use chrono::Utc;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct CalendarEvent {
    pub name: String,
}

#[derive(Debug, Default)]
pub struct CalendarData {
    pub events: HashMap<Utc, CalendarEvent>,
}



fn parseVcalendar(data: &str) -> CalendarData {
    let mut data = CalendarData::default();

    
}