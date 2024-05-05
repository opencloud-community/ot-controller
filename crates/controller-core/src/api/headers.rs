// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[derive(utoipa::ToSchema)]
#[schema(
    example = "<https://api.example.org/resource?page=2>; rel='next', <https://api.example.org/resource?page=5>; rel='last'"
)]
pub struct PageLink(pub String);
