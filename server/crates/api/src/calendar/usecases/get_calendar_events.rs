use crate::shared::usecase::{execute, UseCase};
use crate::{calendar::dtos::CalendarDTO, shared::auth::protect_route};
use crate::{error::NettuError, event::dtos::CalendarEventDTO};

use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_core::{Calendar, CalendarEvent, CalendarView, EventInstance};
use nettu_scheduler_infra::NettuContext;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CalendarPathParams {
    calendar_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimespanParams {
    pub start_ts: i64,
    pub end_ts: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct APIResponse {
    pub calendar: CalendarDTO,
    pub events: Vec<EventWithInstancesDTO>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventWithInstancesDTO {
    pub event: CalendarEventDTO,
    pub instances: Vec<EventInstance>,
}

pub async fn get_calendar_events_controller(
    http_req: HttpRequest,
    query_params: web::Query<TimespanParams>,
    params: web::Path<CalendarPathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, _policy) = protect_route(&http_req, &ctx).await?;

    let usecase = GetCalendarEventsUseCase {
        user_id: user.id,
        calendar_id: params.calendar_id.clone(),
        start_ts: query_params.start_ts,
        end_ts: query_params.end_ts,
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| {
            let res = APIResponse {
                calendar: CalendarDTO::new(&usecase_res.calendar),
                events: usecase_res
                    .events
                    .into_iter()
                    .map(|data| EventWithInstancesDTO {
                        event: CalendarEventDTO::new(&data.event),
                        instances: data.instances,
                    })
                    .collect(),
            };
            HttpResponse::Ok().json(res)
        })
        .map_err(|e| match e {
            UseCaseErrors::InvalidTimespanError => {
                NettuError::BadClientData("The start and end timestamps is invalid".into())
            }
            UseCaseErrors::NotFoundError => NettuError::NotFound(format!(
                "The calendar with id: {}, was not found.",
                params.calendar_id
            )),
        })
}
pub struct GetCalendarEventsUseCase {
    pub calendar_id: String,
    pub user_id: String,
    pub start_ts: i64,
    pub end_ts: i64,
}

pub struct UseCaseResponse {
    calendar: Calendar,
    events: Vec<EventWithInstances>,
}

pub struct EventWithInstances {
    pub event: CalendarEvent,
    pub instances: Vec<EventInstance>,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFoundError,
    InvalidTimespanError,
}

#[async_trait::async_trait(?Send)]
impl UseCase for GetCalendarEventsUseCase {
    type Response = UseCaseResponse;

    type Errors = UseCaseErrors;

    type Context = NettuContext;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let calendar = ctx.repos.calendar_repo.find(&self.calendar_id).await;

        let view = CalendarView::create(self.start_ts, self.end_ts);
        if view.is_err() {
            return Err(UseCaseErrors::InvalidTimespanError);
        }
        let view = view.unwrap();

        match calendar {
            Some(calendar) if calendar.user_id == self.user_id => {
                let events = ctx
                    .repos
                    .event_repo
                    .find_by_calendar(&calendar.id, Some(&view))
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|event| {
                        let instances = event.expand(Some(&view), &calendar.settings);
                        EventWithInstances { event, instances }
                    })
                    // Also it is possible that there are no instances in the expanded event, should remove them
                    .filter(|data| !data.instances.is_empty())
                    .collect();

                Ok(UseCaseResponse { calendar, events })
            }
            _ => Err(UseCaseErrors::NotFoundError),
        }
    }
}