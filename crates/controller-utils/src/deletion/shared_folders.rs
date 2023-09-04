// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use db_storage::events::shared_folders::EventSharedFolder;
use log::Log;
use nextcloud_client::ShareId;
use opentalk_log::{debug, warn};
use settings::Settings;

use super::Error;

/// Delete a list of shared folders from the remote system and the database
pub async fn delete_shared_folders(
    logger: &dyn Log,
    settings: &Settings,
    shared_folders: &[EventSharedFolder],
    fail_on_error: bool,
) -> Result<(), Error> {
    let mut folders_iter = shared_folders.iter();
    let first_folder = folders_iter.next();

    if first_folder.is_none() {
        debug!(log: logger, "No shared folders to delete");
        return Ok(());
    }

    debug!(log: logger, "Reading shared folder settings");
    let shared_folder_settings = settings
        .shared_folder
        .as_ref()
        .ok_or(Error::SharedFoldersNotConfigured)?;

    match shared_folder_settings {
        settings::SharedFolder::Nextcloud {
            url,
            username,
            password,
            ..
        } => {
            debug!(log: logger, "Creating NextCloud client");
            let create_client_result =
                nextcloud_client::Client::new(url.clone(), username.clone(), password.clone());
            let client = match create_client_result {
                Ok(c) => c,
                Err(e) => {
                    warn!(log: logger, "Error creating NextCloud client: {e}");
                    if fail_on_error {
                        return Err(Error::NextcloudClient(e));
                    }
                    return Ok(());
                }
            };
            for shared_folder in first_folder.into_iter().chain(folders_iter) {
                let path = &shared_folder.path;
                debug!(log: logger, "Deleting shared folder from path {path}");
                if path.trim_matches('/').is_empty() {
                    let message = "Preventing recursive deletion of empty shared folder path, this is probably harmful and not intended";
                    if fail_on_error {
                        return Err(Error::Custom(message.to_string()));
                    }
                    warn!(log: logger, "{}", message);
                }
                let user_path = format!("files/{username}/{path}");
                if let Err(e) = client
                    .delete_share(ShareId::from(shared_folder.read_share_id.clone()))
                    .await
                {
                    let message = format!("Could not delete NextCloud read share: {e}");
                    if fail_on_error {
                        return Err(Error::Custom(message));
                    }
                    warn!(log: logger, "{}", message);
                } else {
                    debug!(
                        log: logger,
                        "Deleted read share {:?} from NextCloud", shared_folder.read_share_id
                    );
                }
                if let Err(e) = client
                    .delete_share(ShareId::from(shared_folder.write_share_id.clone()))
                    .await
                {
                    let message = format!("Could not delete NextCloud write share: {e}");
                    if fail_on_error {
                        return Err(Error::Custom(message));
                    }
                    warn!(log: logger, "{}", message);
                } else {
                    debug!(
                        log: logger,
                        "Deleted write share {:?} from NextCloud", shared_folder.write_share_id
                    );
                }
                match client.delete(&user_path).await {
                    Ok(()) => {
                        debug!(
                            log: logger,
                            "Deleted shared folder {user_path:?} from NextCloud"
                        );
                    }
                    Err(nextcloud_client::Error::FileNotFound { file_path, .. }) => {
                        debug!(
                            log: logger,
                            "Path {file_path:?} not found on the NextCloud instance, skipping deletion"
                        );
                    }
                    Err(e) => {
                        let message = format!("Error deleting folder on NextCloud: {e}");
                        if fail_on_error {
                            return Err(Error::Custom(message));
                        }
                        warn!(log: logger, "{}", message);
                    }
                };
            }
            Ok(())
        }
    }
}
