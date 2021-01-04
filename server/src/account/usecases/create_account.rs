use crate::api::Context;
use crate::{
    account::domain::Account,
    shared::usecase::{perform, Usecase},
};
use actix_web::{web, HttpResponse};
use serde::Serialize;

pub async fn create_account_controller(ctx: web::Data<Context>) -> HttpResponse {
    let usecase = CreateAccountUseCase {};
    let res = perform(usecase, &ctx).await;

    match res {
        Ok(json) => HttpResponse::Created().json(json),
        Err(e) => match e {
            UsecaseErrors::StorageError => HttpResponse::InternalServerError().finish(),
        },
    }
}

struct CreateAccountUseCase {}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UseCaseResponse {
    pub account_id: String,
    pub secret_api_key: String,
}

#[derive(Debug)]
enum UsecaseErrors {
    StorageError,
}

#[async_trait::async_trait(?Send)]
impl Usecase for CreateAccountUseCase {
    type Response = UseCaseResponse;

    type Errors = UsecaseErrors;

    type Context = Context;

    async fn perform(&mut self, ctx: &Self::Context) -> Result<Self::Response, Self::Errors> {
        let account = Account::new();
        let res = ctx.repos.account_repo.insert(&account).await;
        match res {
            Ok(_) => Ok(UseCaseResponse {
                account_id: account.id.clone(),
                secret_api_key: account.secret_api_key.clone(),
            }),
            Err(_) => Err(UsecaseErrors::StorageError),
        }
    }
}
