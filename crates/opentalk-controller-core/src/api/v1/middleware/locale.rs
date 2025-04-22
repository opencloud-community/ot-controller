// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_web::dev::ServiceRequest;
use fluent_langneg::{
    convert_vec_str_to_langids_lossy, negotiate_languages, parse_accepted_languages,
    NegotiationStrategy,
};
use opentalk_types_common::users::Language;

pub(super) fn get_request_locale(req: &ServiceRequest) -> Option<Language> {
    // These are the languages supported by the frontend at the moment
    const SUPPORTED_LOCALES: &[&str] = &["de-DE", "en-US"];

    let accepted_languages_header_value = req
        .headers()
        .get("Accept-Language")
        .and_then(|hv| hv.to_str().ok())
        .unwrap_or_default();

    let languages_accepted_by_client = parse_accepted_languages(accepted_languages_header_value);
    let languages_provided_by_service = convert_vec_str_to_langids_lossy(SUPPORTED_LOCALES);
    let languages_available = negotiate_languages(
        &languages_accepted_by_client,
        &languages_provided_by_service,
        None,
        NegotiationStrategy::Filtering,
    );

    languages_available
        .first()
        .and_then(|x| x.to_string().parse().ok())
}
