use crate::{
    error::NettuError,
    shared::usecase::{execute, UseCase},
};
use crate::{
    event::dtos::CalendarEventDTO,
    shared::{
        auth::{protect_route, Permission},
        usecase::{execute_with_policy, PermissionBoundary, UseCaseErrorContainer},
    },
};
use actix_web::{web, HttpRequest, HttpResponse};
use nettu_scheduler_core::CalendarEvent;
use nettu_scheduler_infra::NettuContext;
use serde::Deserialize;

use super::sync_event_reminders::{
    EventOperation, SyncEventRemindersTrigger, SyncEventRemindersUseCase,
};

#[derive(Deserialize)]
pub struct PathParams {
    event_id: String,
}

pub async fn delete_event_controller(
    http_req: HttpRequest,
    path_params: web::Path<PathParams>,
    ctx: web::Data<NettuContext>,
) -> Result<HttpResponse, NettuError> {
    let (user, policy) = protect_route(&http_req, &ctx).await?;

    let usecase = DeleteEventUseCase {
        user_id: user.id.clone(),
        event_id: path_params.event_id.clone(),
    };

    execute_with_policy(usecase, &policy, &ctx)
        .await
        .map(|event| HttpResponse::Ok().json(CalendarEventDTO::new(&event)))
        .map_err(|e| match e {
            UseCaseErrorContainer::Unauthorized(e) => NettuError::Unauthorized(e),
            UseCaseErrorContainer::UseCase(e) => match e {
                UseCaseErrors::NotFound => NettuError::NotFound(format!(
                    "The event with id: {}, was not found",
                    path_params.event_id
                )),
            },
        })
}

pub struct DeleteEventUseCase {
    pub user_id: String,
    pub event_id: String,
}

#[derive(Debug)]
pub enum UseCaseErrors {
    NotFound,
}

#[async_trait::async_trait(?Send)]
impl UseCase for DeleteEventUseCase {
    type Response = CalendarEvent;

    type Errors = UseCaseErrors;

    type Context = NettuContext;

    // TODO: use only one db call
    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let e = ctx.repos.event_repo.find(&self.event_id).await;
        match e {
            Some(event) if event.user_id == self.user_id => {
                ctx.repos.event_repo.delete(&event.id).await;

                let sync_event_reminders = SyncEventRemindersUseCase {
                    request: SyncEventRemindersTrigger::EventModified(
                        &event,
                        EventOperation::Deleted,
                    ),
                };

                // Sideeffect, ignore result
                let _ = execute(sync_event_reminders, ctx).await;

                Ok(event)
            }
            _ => Err(UseCaseErrors::NotFound),
        }
    }
}

impl PermissionBoundary for DeleteEventUseCase {
    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::DeleteCalendarEvent]
    }
}