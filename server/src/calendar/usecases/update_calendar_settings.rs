use crate::shared::usecase::{execute, Usecase};
use crate::{
    api::{Context, NettuError},
    shared::auth::protect_route,
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct UpdateCalendarSettigsPathParams {
    calendar_id: String,
}

#[derive(Deserialize)]
pub struct UpdateCalendarSettingsBody {
    wkst: Option<isize>,
    timezone: Option<String>,
}

pub async fn update_calendar_settings_controller(
    http_req: web::HttpRequest,
    ctx: web::Data<Context>,
    path_params: web::Path<UpdateCalendarSettigsPathParams>,
    body: web::Json<UpdateCalendarSettingsBody>,
) -> Result<HttpResponse, NettuError> {
    let user = protect_route(&http_req, &ctx).await?;

    let usecase = UpdateCalendarSettingsUseCase {
        user_id: user.id,
        calendar_id: path_params.calendar_id.clone(),
        wkst: body.wkst.clone(),
        timezone: body.timezone.clone(),
    };

    execute(usecase, &ctx)
        .await
        .map(|usecase_res| HttpResponse::Ok().json(usecase_res))
        .map_err(|e| match e {
            UseCaseErrors::StorageError => NettuError::InternalError,
            UseCaseErrors::CalendarNotFoundError => {
                NettuError::NotFound("The calendar was not found.".into())
            }
            UseCaseErrors::InvalidSettings(err) => NettuError::BadClientData(format!(
                "Bad calendar settings provided. Error message: {}",
                err
            )),
        })
}

struct UpdateCalendarSettingsUseCase {
    pub user_id: String,
    pub calendar_id: String,
    pub wkst: Option<isize>,
    pub timezone: Option<String>,
}

#[derive(Debug)]
enum UseCaseErrors {
    CalendarNotFoundError,
    StorageError,
    InvalidSettings(String),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UseCaseRes {}

#[async_trait::async_trait(?Send)]
impl Usecase for UpdateCalendarSettingsUseCase {
    type Response = UseCaseRes;

    type Errors = UseCaseErrors;

    type Context = Context;

    async fn execute(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let mut calendar = match ctx.repos.calendar_repo.find(&self.calendar_id).await {
            Some(cal) if cal.user_id == self.user_id => cal,
            _ => return Err(UseCaseErrors::CalendarNotFoundError),
        };

        if let Some(wkst) = self.wkst {
            if !calendar.settings.set_wkst(wkst) {
                return Err(UseCaseErrors::InvalidSettings(format!(
                    "Invalid wkst property: {}, must be between 0 and 6",
                    wkst
                )));
            }
        }

        if let Some(timezone) = &self.timezone {
            if !calendar.settings.set_timezone(timezone) {
                return Err(UseCaseErrors::InvalidSettings(format!(
                    "Invalid timezone property: {}, must be a valid IANA Timezone string",
                    timezone
                )));
            }
        }

        let repo_res = ctx.repos.calendar_repo.save(&calendar).await;
        match repo_res {
            Ok(_) => Ok(UseCaseRes {}),
            Err(_) => Err(UseCaseErrors::StorageError),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        calendar::domain::Calendar,
        event::domain::event::{CalendarEvent, RRuleOptions},
    };

    use super::*;

    #[actix_web::main]
    #[test]
    async fn it_rejects_invalid_wkst() {
        let ctx = Context::create_inmemory();
        let user_id = "1".to_string();
        let calendar = Calendar::new(&user_id);
        ctx.repos.calendar_repo.insert(&calendar).await.unwrap();

        let mut usecase = UpdateCalendarSettingsUseCase {
            calendar_id: calendar.id.into(),
            user_id: user_id.into(),
            wkst: Some(20),
            timezone: None,
        };
        let res = usecase.execute(&ctx).await;
        assert!(res.is_err());
    }

    #[actix_web::main]
    #[test]
    async fn it_update_settings_with_valid_wkst() {
        let ctx = Context::create_inmemory();
        let user_id = "1".to_string();
        let calendar = Calendar::new(&user_id);
        ctx.repos.calendar_repo.insert(&calendar).await.unwrap();

        assert_eq!(calendar.settings.wkst, 0);
        let new_wkst = 3;
        let mut usecase = UpdateCalendarSettingsUseCase {
            calendar_id: calendar.id.clone(),
            user_id: user_id.into(),
            wkst: Some(new_wkst),
            timezone: None,
        };
        let res = usecase.execute(&ctx).await;
        assert!(res.is_ok());

        // Check that calendar settings have been updated
        let calendar = ctx.repos.calendar_repo.find(&calendar.id).await.unwrap();
        assert_eq!(calendar.settings.wkst, new_wkst);
    }
}