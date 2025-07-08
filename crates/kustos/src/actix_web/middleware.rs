// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Actix-web middleware based on <https://github.com/casbin-rs/actix-casbin-auth>
use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    pin::Pin,
    rc::Rc,
    str::FromStr,
    sync::Arc,
    task::{Context, Poll},
};

use actix_web::{
    Error, HttpMessage, HttpResponse, Result,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use casbin::CoreApi;
use futures::{
    Future,
    future::{Ready, ok},
};
use itertools::Itertools;
use kustos_shared::{access::AccessMethod, subject::PolicyUser};
use snafu::Report;
use tokio::sync::RwLock;
use tracing_futures::Instrument;

use crate::{
    actix_web::{Invite, User},
    internal::{block, synced_enforcer::SyncedEnforcer},
    policy::{InvitePolicy, UserPolicy},
    subject::PolicyInvite,
};

#[derive(Clone)]
pub struct KustosService {
    strip_versioned_path: bool,
    enforcer: Arc<RwLock<SyncedEnforcer>>,
}

impl KustosService {
    pub async fn new(enforcer: Arc<RwLock<SyncedEnforcer>>, strip_versioned_path: bool) -> Self {
        KustosService {
            strip_versioned_path,
            enforcer,
        }
    }

    pub fn get_enforcer(&self) -> Arc<RwLock<SyncedEnforcer>> {
        self.enforcer.clone()
    }

    pub fn set_enforcer(&self, e: Arc<RwLock<SyncedEnforcer>>) -> KustosService {
        KustosService {
            enforcer: e,
            strip_versioned_path: self.strip_versioned_path,
        }
    }
}

impl<S> Transform<S, ServiceRequest> for KustosService
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = KustosMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(KustosMiddleware {
            strip_versioned_path: self.strip_versioned_path,
            enforcer: self.enforcer.clone(),
            service: Rc::new(RefCell::new(service)),
        })
    }
}

impl Deref for KustosService {
    type Target = Arc<RwLock<SyncedEnforcer>>;

    fn deref(&self) -> &Self::Target {
        &self.enforcer
    }
}

impl DerefMut for KustosService {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.enforcer
    }
}

type ResultFuture<O, E> = Pin<Box<dyn Future<Output = Result<O, E>>>>;

pub struct KustosMiddleware<S> {
    strip_versioned_path: bool,
    service: Rc<RefCell<S>>,
    enforcer: Arc<RwLock<SyncedEnforcer>>,
}

impl<S> Service<ServiceRequest> for KustosMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = ResultFuture<Self::Response, Self::Error>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let cloned_enforcer = self.enforcer.clone();
        let srv = self.service.clone();
        let strip_versioned_path = self.strip_versioned_path;
        Box::pin(
            async move {
                let path = if strip_versioned_path {
                    get_unprefixed_path(req.path())?
                } else {
                    req.path().to_string()
                };
                let action = req.method().as_str();
                let action = match AccessMethod::from_str(action) {
                    Ok(action) => action,
                    Err(e) => {
                        log::error!("Invalid method {}", Report::from_error(e));
                        return Ok(req.into_response(HttpResponse::Unauthorized().finish()));
                    }
                };

                let span = tracing::Span::current();

                let optional_user = req.extensions().get::<User>().cloned();
                let optional_invite = req.extensions().get::<Invite>().cloned();

                let enforce_result = match (optional_user, optional_invite) {
                    (Some(user), _) => {
                        let subject: PolicyUser = user.0.into();
                        block(move || {
                            let lock = cloned_enforcer.blocking_read();

                            lock.enforce(UserPolicy::new(subject, path, action))
                        })
                        .await
                    }
                    (_, Some(invite)) => {
                        let subject: PolicyInvite = invite.0.into();
                        block(move || {
                            let lock = cloned_enforcer.blocking_read();

                            lock.enforce(InvitePolicy::new(subject, path, action))
                        })
                        .await
                    }
                    _ => {
                        // TODO(r.floren) This way we might leak the existence of resources. We might want to use NotFound in some cases.
                        log::trace!("No user found. Responding with 401");
                        return Ok(req.into_response(HttpResponse::Unauthorized().finish()));
                    }
                };

                match enforce_result {
                    Ok(Ok(true)) => {
                        span.record("enforcement", true);
                        srv.call(req).await
                    }
                    Ok(Ok(false)) => {
                        span.record("enforcement", false);
                        Ok(req.into_response(HttpResponse::Forbidden().finish()))
                    }
                    Ok(Err(e)) => {
                        log::error!("Enforce error {}", Report::from_error(e));
                        Ok(req.into_response(HttpResponse::BadGateway().finish()))
                    }
                    Err(_) => {
                        log::error!("Enforcer panicked");
                        Ok(req.into_response(HttpResponse::InternalServerError().finish()))
                    }
                }
            }
            .instrument(tracing::debug_span!(
                "CasbinMiddleware::async::call",
                enforcement = tracing::field::Empty
            )),
        )
    }
}

/// Returns the unversioned path in case the input path is versioned (i.e. starts with /v{number})
///
/// Way to avoid a regex here
fn get_unprefixed_path(input_path: &str) -> Result<String> {
    let mut segments = input_path.split('/');

    // skip the first segment as its an empty string before the first '/'
    if input_path.starts_with('/') {
        segments.next();
    }

    if let Some(segment) = segments.next() {
        let mut chars = segment.chars();

        if let Some('v') = chars.next() {
            if chars.all(char::is_numeric) {
                // TODO(kbalt): use Split::as_str() when stabilized
                // see https://github.com/rust-lang/rust/issues/77998
                // return Ok(format!("/{}", segments.as_str()));

                return Ok(format!("/{}", segments.join("/")));
            }
        }
    }

    Ok(input_path.to_owned())
}
