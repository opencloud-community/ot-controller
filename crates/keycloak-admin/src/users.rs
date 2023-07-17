// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::Result;
use crate::KeycloakAdminClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
enum ParseResult {
    User(User),
    Failure {},
}

impl From<ParseResult> for Option<User> {
    fn from(result: ParseResult) -> Self {
        match result {
            ParseResult::User(user) => Some(user),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub username: String,
    pub enabled: bool,
    pub email_verified: bool,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    #[serde(default)]
    pub attributes: Attributes,
}

impl User {
    fn is_in_tenant(&self, tenant_id: &str) -> bool {
        self.attributes.tenant_id.iter().any(|t| t == tenant_id)
    }
}

#[derive(Default, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Attributes {
    #[serde(default)]
    pub tenant_id: Vec<String>,
}

#[derive(Serialize)]
struct SearchQuery<'s> {
    search: &'s str,
    max: i32,
}

#[derive(Serialize)]
struct VerifyEmailQuery<'s> {
    email: &'s str,
    exact: bool,
}

impl KeycloakAdminClient {
    /// Query keycloak for users using the given search string
    pub async fn search_user(&self, search_str: &str) -> Result<Vec<User>> {
        let url = self.url(["admin", "realms", &self.realm, "users"])?;

        let query = SearchQuery {
            search: search_str,
            max: 5,
        };

        let response = self
            .send_authorized(move |c| c.get(url.clone()).query(&query))
            .await?;

        let found_results = response.json::<Vec<ParseResult>>().await?;
        let found_results_amount = found_results.len();
        let found_users: Vec<_> = found_results
            .into_iter()
            .filter_map(Option::<User>::from)
            .collect();

        if found_results_amount != found_users.len() {
            log::warn!("User JSON object from KeyCloak is missing at least one field.");
        }

        Ok(found_users)
    }

    /// Query keycloak for users using the given search string, filtered by the tenant_id
    pub async fn search_user_filtered(
        &self,
        tenant_id: &str,
        search_str: &str,
    ) -> Result<Vec<User>> {
        let url = self.url(["admin", "realms", &self.realm, "users"])?;

        // TODO: Fix this code once https://github.com/keycloak/keycloak/issues/16687 is resolved.
        // Currently we let keycloak give us 100 users matching the search_str and then filter by the tenant_id.
        // Ideally we want keycloak to filter these out for us. In a larger user base with more tenants this will
        // will be insufficient to provide a good auto-completion, since more than 100 users outside of the searched
        // tenant might match.
        let query = SearchQuery {
            search: search_str,
            max: 100,
        };

        let response = self
            .send_authorized(move |c| c.get(url.clone()).query(&query))
            .await?;

        let found_results = response.json::<Vec<ParseResult>>().await?;
        let found_results_amount = found_results.len();
        let found_users: Vec<_> = found_results
            .into_iter()
            .filter_map(Option::<User>::from)
            .filter(|user| user.is_in_tenant(tenant_id))
            .take(5)
            .collect::<Vec<User>>();

        if found_results_amount != found_users.len() {
            log::warn!("User JSON object from KeyCloak is missing at least one field.");
        }

        Ok(found_users)
    }

    /// Query keycloak to get the first user that matches the given email
    pub async fn get_user_for_email(
        &self,
        tenant_id: Option<&str>,
        email: &str,
    ) -> Result<Option<User>> {
        let url = self.url(["admin", "realms", &self.realm, "users"])?;

        let query = VerifyEmailQuery { email, exact: true };

        let response = self
            .send_authorized(move |c| c.get(url.clone()).query(&query))
            .await?;

        let found_users: Vec<User> = response.json().await?;
        let first_matching_user = found_users.iter().find(|user| {
            let tenant_matches = tenant_id.is_none() || user.is_in_tenant(tenant_id.unwrap());
            tenant_matches && user.email == email
        });

        if let Some(user) = first_matching_user {
            return Ok(Some((*user).clone()));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn deserialize_user() {
        use super::*;

        let json_with_email = serde_json::json!({
            "id": "someid",
            "username": "someuser",
            "enabled": true,
            "emailVerified": true,
            "firstName": "somefirstname",
            "lastName": "somelastname",
            "email": "someemail",
        });

        let json_without_email = serde_json::json!({
            "id": "someid",
            "username": "someuser",
            "enabled": true,
            "emailVerified": true,
            "firstName": "somefirstname",
            "lastName": "somelastname",
        });

        let parse_result_user: ParseResult = serde_json::from_value(json_with_email).unwrap();
        let parse_result_failure: ParseResult = serde_json::from_value(json_without_email).unwrap();

        assert!(matches!(parse_result_user, ParseResult::User(_)));
        assert!(matches!(parse_result_failure, ParseResult::Failure {}));
    }
}
