// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use core::fmt;
use std::borrow::Cow;

use http0::StatusCode;
use itertools::Itertools;

use super::{
    AuthenticationError, ErrorBody, StandardErrorBody, ValidationErrorBody, ValidationErrorEntry,
};
use crate::api::v1::rooms::StartRoomError;
#[allow(unused_imports)]
use crate::imports::*;

/// The default REST API error
///
/// Can be build via the associated functions to represent various HTTP errors. Each
/// HTTP error has their default error code and message that get send in a JSON body.
/// The error code and message can be overwritten when creating an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    /// The HTTP status code of the error
    pub status: StatusCode,

    /// An optional authentication header value
    pub www_authenticate: Option<AuthenticationError>,

    /// The body of the error
    pub body: ErrorBody,
}

impl ApiError {
    fn new_standard<T>(status: StatusCode, code: T, message: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        Self {
            status,
            www_authenticate: None,
            body: ErrorBody::Standard(StandardErrorBody {
                code: code.into(),
                message: message.into(),
            }),
        }
    }

    /// Override the default code for an error
    pub fn with_code<T>(mut self, code: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        match &mut self.body {
            ErrorBody::Standard(std) => std.code = code.into(),
            ErrorBody::Validation(val) => val.code = code.into(),
        }

        self
    }

    /// Override the default message for an error
    pub fn with_message<T>(mut self, message: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        match &mut self.body {
            ErrorBody::Standard(std) => std.message = message.into(),
            ErrorBody::Validation(val) => val.message = message.into(),
        }

        self
    }

    /// Add the www_authenticate header to this error
    pub fn with_www_authenticate(mut self, authentication_error: AuthenticationError) -> Self {
        self.www_authenticate = Some(authentication_error);

        self
    }

    /// Create a new 400 Bad Request error
    pub fn bad_request() -> Self {
        Self::new_standard(
            StatusCode::BAD_REQUEST,
            "bad_request",
            "Invalid request due to malformed syntax",
        )
    }

    /// Create a new 401 Unauthorized error
    pub fn unauthorized() -> Self {
        Self::new_standard(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "Authentication failed",
        )
    }

    /// Create a new 403 Forbidden error
    pub fn forbidden() -> Self {
        Self::new_standard(
            StatusCode::FORBIDDEN,
            "forbidden",
            "Access to the requested resource is forbidden",
        )
    }

    /// Create a new 404 Not Found error
    pub fn not_found() -> Self {
        Self::new_standard(
            StatusCode::NOT_FOUND,
            "not_found",
            "A requested resource could not be found",
        )
    }

    /// Create a new 409 Conflict error
    pub fn conflict() -> Self {
        Self::new_standard(
            StatusCode::CONFLICT,
            "conflict",
            "The request conflicts with the state of the resource",
        )
    }

    /// Create a new 422 Unprocessable Entity error
    ///
    /// see [`Self::unprocessable_entities()`]
    pub fn unprocessable_entity() -> Self {
        Self::unprocessable_entities::<ValidationErrorEntry, _>([])
    }

    /// Create a new 422 Unprocessable Entity error
    ///
    /// This error is normally created from [`ValidationErrors`] from the validator crate.
    /// The JSON body for this error additionally contains a list of errors for each invalid field.
    pub fn unprocessable_entities<T, I>(errors: I) -> Self
    where
        T: Into<ValidationErrorEntry>,
        I: IntoIterator<Item = T>,
    {
        let errors = errors.into_iter().map(|entry| entry.into()).collect();

        let validation_body = ValidationErrorBody::new(
            "validation_failed",
            "Some provided values are invalid",
            errors,
        );

        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            www_authenticate: None,
            body: ErrorBody::Validation(validation_body),
        }
    }

    /// Create a new 500 Internal Server Error
    pub fn internal() -> Self {
        Self::new_standard(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_server_error",
            "An internal server error occurred",
        )
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.body {
            ErrorBody::Standard(StandardErrorBody { code, message }) => {
                write!(
                    f,
                    "status={}, code={}, message={}",
                    self.status, code, message
                )
            }
            ErrorBody::Validation(ValidationErrorBody {
                code,
                message,
                errors,
            }) => {
                write!(
                    f,
                    "status={}, code={}, message={}, errors={}",
                    self.status,
                    code,
                    message,
                    serde_json::to_string(errors)
                        .unwrap_or_else(|_| "unserializable errors".to_string())
                )
            }
        }
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let mut builder = axum::response::Response::builder();

        builder = builder.status(self.status.as_u16());

        builder = builder.header(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("text/json; charset=utf-8"),
        );

        let builder = if let Some(www_authenticate) = self.www_authenticate {
            builder.header(
                http::header::WWW_AUTHENTICATE,
                www_authenticate.header_value(),
            )
        } else {
            builder
        };

        let body = serde_json::to_string(&self.body).expect("Unable to serialize API error body");

        builder
            .body(axum::body::Body::new(body))
            .expect("Failed to build axum Response ")
    }
}

#[cfg(feature = "actix")]
impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        self.status
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_http::body::BoxBody> {
        let mut response = actix_web::HttpResponse::new(self.status_code());

        let _ = response.headers_mut().insert(
            http0::header::CONTENT_TYPE,
            actix_http::header::HeaderValue::from_static("text/json; charset=utf-8"),
        );

        if let Some(www_authenticate) = self.www_authenticate {
            let _ = response.headers_mut().insert(
                http0::header::WWW_AUTHENTICATE,
                www_authenticate
                    .header_value()
                    .try_into()
                    .expect("Unable to create www-authenticate bearer header-value"),
            );
        }

        let body = serde_json::to_string(&self.body).expect("Unable to serialize API error body");

        response.set_body(actix_http::body::BoxBody::new(body))
    }
}

#[cfg(feature = "actix")]
impl From<actix_web::Error> for ApiError {
    fn from(value: actix_web::Error) -> Self {
        log::error!("REST API threw internal error from actix web error: {value}");
        Self::internal()
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::result::Error> for ApiError {
    fn from(e: diesel::result::Error) -> Self {
        log::error!(
            "REST API threw internal error from Diesel error: {}",
            snafu::Report::from_error(e)
        );
        Self::internal()
    }
}

impl From<StartRoomError> for ApiError {
    fn from(start_room_error: StartRoomError) -> Self {
        match start_room_error {
            StartRoomError::WrongRoomPassword => Self::unauthorized()
                .with_code(StartRoomError::WrongRoomPassword.as_ref())
                .with_message("The provided password does not match the room password"),

            StartRoomError::NoBreakoutRooms => Self::bad_request()
                .with_code(StartRoomError::NoBreakoutRooms.as_ref())
                .with_message("The requested room has no breakout rooms"),

            StartRoomError::InvalidBreakoutRoomId => Self::bad_request()
                .with_code(StartRoomError::InvalidBreakoutRoomId.as_ref())
                .with_message("The provided breakout room ID is invalid"),

            StartRoomError::BannedFromRoom => Self::forbidden()
                .with_code(StartRoomError::BannedFromRoom.as_ref())
                .with_message("This user has been banned from entering this room"),
        }
    }
}

impl From<snafu::Whatever> for ApiError {
    fn from(value: snafu::Whatever) -> Self {
        log::error!("REST API threw generic internal error: {value:?}");
        ApiError::internal()
    }
}

#[cfg(feature = "backend")]
impl From<opentalk_database::DatabaseError> for ApiError {
    fn from(value: opentalk_database::DatabaseError) -> ApiError {
        use opentalk_database::DatabaseError;

        match value {
            DatabaseError::NotFound => ApiError::not_found(),
            DatabaseError::DieselError {
                source:
                    diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                        _,
                    ),
            } => ApiError::conflict(),
            e => {
                log::error!(
                    "REST API threw internal error from database error: {}",
                    snafu::Report::from_error(e)
                );
                ApiError::internal()
            }
        }
    }
}

#[cfg(feature = "backend")]
impl From<opentalk_cache::CacheError> for ApiError {
    fn from(value: opentalk_cache::CacheError) -> Self {
        log::error!("REST API threw internal error while writing/reading cache: {value}");
        ApiError::internal()
    }
}

impl From<ValidationErrors> for ApiError {
    /// Creates a 422 Unprocessable entity response from the [`ValidationErrors`]
    ///
    /// Note:
    ///
    /// Each validation error is mapped to a field. When we encounter a validation error on a nested struct, we
    /// assume the struct was perceived flattened in it's JSON representation and do not distinguish between nested and
    /// non-nested fields. This may lead to ambiguous field mappings when receiving invalid fields for actually
    /// nested fields.
    ///
    /// We currently have no feasible way to identify correct the JSON representation.
    ///
    /// Example for this misleading behavior:
    ///
    /// Assuming the request body has the following structure:
    /// ```json
    /// {
    ///     "name": "foo",
    ///     "age": 30,
    ///     "nested":
    ///     {
    ///         "name": "bar",
    ///         "age": 24
    ///     }
    /// }
    /// ```
    ///
    ///  Assuming one of the `name` fields is invalid, the resulting validation error would look something like this:
    ///
    /// ```json
    /// {
    ///     "code": "validation_failed",
    ///     "message": "Some provided values are invalid",
    ///     "errors":
    ///     [
    ///         {
    ///             "field": "name",
    ///             "code": "invalid_value"
    ///         }
    ///     ]
    /// }
    /// ```
    ///
    /// The sender has no way to identify which of the `name` fields is invalid, except manually reviewing the values and
    /// reading the API docs.
    fn from(validation_errors: ValidationErrors) -> Self {
        let mut entries = Vec::with_capacity(validation_errors.errors().len());

        collect_validation_errors(validation_errors, &mut entries);

        Self::unprocessable_entities(entries)
    }
}

/// Convert [`ValidationErrors`] into multiple [`ValidationErrorEntries`](ValidationErrorEntry) and collect them in `entries`
fn collect_validation_errors(
    validation_errors: ValidationErrors,
    entries: &mut Vec<ValidationErrorEntry>,
) {
    let errors = validation_errors.into_errors();

    for (field, error_kind) in errors {
        let field = match field {
            "__all__" => None,
            field => Some(field.into()),
        };

        match error_kind {
            validator::ValidationErrorsKind::Field(v) => {
                for error in v {
                    let code = convert_validation_code(&error.code);

                    entries.push(ValidationErrorEntry {
                        field: field.clone(),
                        code: Cow::Borrowed(code),
                        message: error.message,
                    });
                }
            }
            validator::ValidationErrorsKind::Struct(inner_errors) => {
                // Assume all fields were flattened when we encounter a struct level validation error
                collect_validation_errors(*inner_errors.to_owned(), entries);
            }
            validator::ValidationErrorsKind::List(list) => {
                let invalid_indexes = list.iter().map(|(idx, ..)| idx).take(15).join(", ");

                let message = format!("Invalid values at index {invalid_indexes}");

                entries.push(ValidationErrorEntry {
                    field,
                    code: "invalid_values".into(),
                    message: Some(Cow::Owned(message)),
                })
            }
        };
    }
}

fn convert_validation_code(code: &str) -> &'static str {
    match code {
        "email" => super::ERROR_CODE_INVALID_EMAIL,
        "url" => super::ERROR_CODE_INVALID_URL,
        "length" => super::ERROR_CODE_INVALID_LENGTH,
        "range" => super::ERROR_CODE_OUT_OF_RANGE,
        "required" => super::ERROR_CODE_VALUE_REQUIRED,
        "empty" => super::ERROR_CODE_MISSING_VALUE,
        _ => super::ERROR_CODE_INVALID_VALUE,
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use validator::Validate;

    use super::*;

    #[derive(Debug, Validate)]
    struct ValidationTester {
        #[validate(email)]
        mail: String,
        #[validate(url(message = "This would be a message"))]
        url: String,
        #[validate(length(max = 5))]
        length: String,
        #[validate(range(min = 5, max = 10))]
        range: usize,
        #[validate(required)]
        required: Option<bool>,
        #[validate(nested)]
        inner_struct: InnerValidationTester,
    }

    #[derive(Debug, Validate)]
    struct InnerValidationTester {
        #[validate(range(max = 2))]
        another_range: usize,
    }

    #[test]
    fn api_validation_error() {
        let tester = ValidationTester {
            mail: "not_a_mail".into(),
            url: "not_a_url".into(),
            length: "looong".into(),
            range: 11,
            required: None,
            inner_struct: InnerValidationTester { another_range: 3 },
        };

        let mut api_error = match tester.validate() {
            Ok(_) => panic!("Validation should fail"),
            Err(err) => ApiError::from(err),
        };

        match &mut api_error.body {
            ErrorBody::Standard(_) => panic!("Expected validation error body"),
            ErrorBody::Validation(val) => val.errors.sort_by(|a, b| a.field.cmp(&b.field)),
        }

        assert_eq!(
            serde_json::to_value(api_error.body).unwrap(),
            json!({
                "code": "validation_failed",
                "message": "Some provided values are invalid",
                "errors": [
                  {
                    "field": "another_range",
                    "code": "out_of_range"
                  },
                  {
                    "field": "length",
                    "code": "invalid_length"
                  },
                  {
                    "field": "mail",
                    "code": "invalid_email"
                  },
                  {
                    "field": "range",
                    "code": "out_of_range"
                  },
                  {
                    "field": "required",
                    "code": "value_required",
                  },
                  {
                    "field": "url",
                    "code": "invalid_url",
                    "message": "This would be a message"
                  }
                ]
            })
        );
    }

    #[test]
    fn api_error_with_code() {
        let error = ApiError::not_found().with_code("custom_code");

        assert_eq!(
            serde_json::to_value(error.body).unwrap(),
            json!({
                "code": "custom_code",
                "message": "A requested resource could not be found"
            })
        );
    }

    #[test]
    fn api_error_with_message() {
        let error = ApiError::not_found().with_message("A custom message");

        assert_eq!(
            serde_json::to_value(error.body).unwrap(),
            json!({
                "code": "not_found",
                "message": "A custom message"
            })
        );
    }
}
