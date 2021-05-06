use chrono::{DateTime, TimeZone, Utc};
use nettu_scheduler_domain::CalendarEvent;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::log::warn;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleCalendarEventDateTime {
    date_time: GoogleDateTime,
    time_zone: String,
}

impl GoogleCalendarEventDateTime {
    pub fn new(date_time_millis: i64) -> Self {
        Self {
            date_time: GoogleDateTime::from_timestamp_millis(date_time_millis),
            time_zone: String::from("UTC"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleCalendarEvent {
    start: GoogleCalendarEventDateTime,
    end: GoogleCalendarEventDateTime,
    summary: String,
    description: String,
    recurrence: Vec<String>,
}

impl From<CalendarEvent> for GoogleCalendarEvent {
    fn from(e: CalendarEvent) -> Self {
        Self {
            description: format!(""),
            summary: format!(""),
            start: GoogleCalendarEventDateTime::new(e.start_ts),
            end: GoogleCalendarEventDateTime::new(e.end_ts),
            // Recurrence sync not supported yet
            recurrence: vec![],
        }
    }
}

pub struct GoogleCalendarRestApi {
    client: Client,
    access_token: String,
}

impl GoogleCalendarRestApi {
    pub fn new(access_token: String) -> Self {
        let client = Client::new();

        Self {
            client,
            access_token,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleDateTime(String);

impl GoogleDateTime {
    pub fn from_timestamp_millis(timestamp: i64) -> Self {
        let datetime_str = Utc.timestamp_millis(timestamp).to_rfc3339();
        Self(datetime_str)
    }

    pub fn get_timestamp_millis(&self) -> i64 {
        DateTime::parse_from_rfc3339(&self.0)
            .expect("Inner string to always be valid RFC3339 string")
            .timestamp_millis()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeBusyCalendarResponse {
    pub busy: Vec<FreeBusyTimeSpanResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeBusyTimeSpanResponse {
    pub start: GoogleDateTime,
    pub end: GoogleDateTime,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeBusyResponse {
    kind: String,
    time_min: GoogleDateTime,
    time_max: GoogleDateTime,
    pub calendars: HashMap<String, FreeBusyCalendarResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeBusyCalendar {
    pub id: String,
}

impl FreeBusyCalendar {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeBusyRequest {
    pub time_min: GoogleDateTime,
    pub time_max: GoogleDateTime,
    pub time_zone: String,
    pub items: Vec<FreeBusyCalendar>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCalendarsResponse {
    kind: String,
    etag: GoogleDateTime,
    pub items: Vec<GoogleCalendarListEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleCalendarListEntry {
    pub id: String,
    summary: String,
    description: String,
    location: String,
    time_zone: String,
    summary_override: String,
    color_id: String,
    background_color: String,
    foreground_color: String,
    hidden: bool,
    selected: bool,
    access_role: GoogleCalendarAccessRole,
    primary: bool,
    deleted: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GoogleCalendarAccessRole {
    Owner,
    Writer,
    Reader,
    FreeBusyReader,
}

const GOOGLE_API_BASE_URL: &str = "https://www.googleapis.com/calendar/v3";

impl GoogleCalendarRestApi {
    async fn post<T: for<'de> Deserialize<'de>>(
        &self,
        body: &impl Serialize,
        path: String,
    ) -> Result<T, ()> {
        match self
            .client
            .post(&format!("{}/{}", GOOGLE_API_BASE_URL, path))
            .header("authorization", format!("Bearer: {}", self.access_token))
            .json(body)
            .send()
            .await
        {
            Ok(res) => res.json::<T>().await.map_err(|e| {
                warn!("Error: {:?}", e);
                ()
            }),
            Err(_) => Err(()),
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: String) -> Result<T, ()> {
        match self
            .client
            .get(&format!("{}/{}", GOOGLE_API_BASE_URL, path))
            .header("authorization", format!("Bearer: {}", self.access_token))
            .send()
            .await
        {
            Ok(res) => res.json::<T>().await.map_err(|e| {
                warn!("Error: {:?}", e);
                ()
            }),
            Err(_) => Err(()),
        }
    }

    async fn delete<T: for<'de> Deserialize<'de>>(&self, path: String) -> Result<T, ()> {
        match self
            .client
            .delete(&format!("{}/{}", GOOGLE_API_BASE_URL, path))
            .header("authorization", format!("Bearer: {}", self.access_token))
            .send()
            .await
        {
            Ok(res) => res.json::<T>().await.map_err(|e| {
                warn!("Error: {:?}", e);
                ()
            }),
            Err(_) => Err(()),
        }
    }

    pub async fn freebusy(&self, body: &FreeBusyRequest) -> Result<FreeBusyResponse, ()> {
        self.post(body, "freebusy".into()).await
    }

    pub async fn insert(
        &self,
        calendar_id: String,
        body: &GoogleCalendarEvent,
    ) -> Result<GoogleCalendarEvent, ()> {
        self.post(body, format!("{}/events", calendar_id)).await
    }

    pub async fn remove(&self, calendar_id: String, event_id: String) -> Result<(), ()> {
        self.delete(format!("{}/events/{}", calendar_id, event_id))
            .await
    }

    pub async fn list(
        &self,
        min_access_role: GoogleCalendarAccessRole,
    ) -> Result<ListCalendarsResponse, ()> {
        self.get(format!(
            "users/me/calendarList?minAccessRole={:?}",
            min_access_role
        ))
        .await
    }
}
