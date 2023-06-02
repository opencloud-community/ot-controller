// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Try to match a request action (e.g. `GET`) to a policy's allowed actions (e.g. `GET|POST|PUT`)
/// Used instead of regexMatch to improve enforce performance
pub(crate) fn act_match(request: &str, policy: &str) -> bool {
    policy
        .split('|')
        .any(|policy_segment| policy_segment == request)
}

/// Match a request against a policy
///
/// This supports wildcard syntax at any segment in the policy, segments are delimited by slashes.
///
/// Examples of supported policy syntax:
///  - /resource/123
///  - /resource/*
///  - /resource/*/subresource
///  - /resource/*/subresource/*
///  - /resource/*/subresource/*/ - The request must also contain a trailing slash to match this
///
/// # Example
///
/// ## Basic example
///
/// ```ignore
/// // Define a policy which grants access to the resource with an id 123
/// let policy = "/resources/123";
/// // The requester wants to access resource with the id 123
/// let request = "/resources/123";
///
/// let result = obj_match(request, policy); // matches!
/// # assert!(result);
///
/// // The requester wants to access resource with the id 511
/// let request = "/resources/511";
/// let result = obj_match(request, policy); // does not match
/// # assert!(!result);
/// ```
///
/// ## A wildcard example
///
/// ```ignore
/// // Define a policy which grants access to all resources
/// let policy = "/resources/*";
/// // The requester wants to access resource with the id 123
/// let request = "/resources/123";
///
/// let result = obj_match(request, policy); // matches!
/// # assert!(result);
///
/// // Policies may also have more than one wildcard and not just at the end
/// let policy = "/resources/*/subresource/*";
/// let request= "/resources/123/subresource/321";
///
/// let result = obj_match(request, policy); // matches!
/// # assert!(result);
/// ```
pub fn obj_match(request: &str, policy: &str) -> bool {
    // If the policy is larger than the request, there isn't a way a wildcard (*) would match, so this never matches
    if policy.len() > request.len() {
        return false;
    }

    let mut request_iter = request.chars();
    let mut policy_iter = policy.chars();

    loop {
        // Iterate all bytes and compare them
        match (request_iter.next(), policy_iter.next()) {
            (Some(request_byte), Some(policy_byte)) => {
                if request_byte == policy_byte {
                    // If the bytes are the same just continue to the next one
                    continue;
                } else if policy_byte == '*' {
                    // If the the policy byte is a wildcard (*) we skip every request byte until we hit a slash (/)
                    skip_until_next_slash(&mut request_iter);

                    // Skip over the policy slash (/) that MUST come after the wildcard (*)
                    let _ = policy_iter.next();
                } else {
                    // Fallthrough, nothing matches :(
                    return false;
                }
            }
            // Both are out of bytes and didn't fail anywhere
            (None, None) => return true,
            _ => return false,
        }
    }
}

fn skip_until_next_slash<'b, I>(iter: &mut I)
where
    I: Iterator<Item = char> + 'b,
{
    for c in iter {
        if c == '/' {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obj_match() {
        assert!(obj_match("/rooms/123", "/rooms/123"));
        assert!(!obj_match("/rooms/123", "/rooms/321"));
        assert!(obj_match("/rooms/123", "/rooms/*"));
        assert!(obj_match("/rooms/123/", "/rooms/*/"));
        assert!(obj_match("/rooms/123/", "/rooms/123/"));
        assert!(obj_match("/rooms/123/start", "/rooms/*/start"));
        assert!(!obj_match("/rooms/123", "/rooms/*/start"));
        assert!(!obj_match("/rooms/123/start", "/rooms/*"));
    }

    #[test]
    fn test_act_match() {
        assert!(act_match("GET", "GET|POST|PUT"));
        assert!(act_match("POST", "GET|POST|PUT"));
        assert!(act_match("PUT", "GET|POST|PUT"));
        assert!(!act_match("DELETE", "GET|POST|PUT"));
    }
}
