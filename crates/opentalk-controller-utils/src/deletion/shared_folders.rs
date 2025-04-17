// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use log::Log;
use opentalk_controller_settings::Settings;
use opentalk_db_storage::events::shared_folders::EventSharedFolder;
use opentalk_log::{debug, warn};
use opentalk_nextcloud_client::ShareId;
use snafu::{whatever, Report};

use super::error::Error;

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
        opentalk_controller_settings::settings_file::SharedFolder::Nextcloud {
            url,
            username,
            password,
            ..
        } => {
            debug!(log: logger, "Creating NextCloud client");
            let create_client_result = opentalk_nextcloud_client::Client::new(
                url.clone(),
                username.clone(),
                password.clone(),
            );
            let client = match create_client_result {
                Ok(c) => c,
                Err(e) => {
                    let msg = "Error creating NextCloud client";
                    warn!(log: logger, "{msg}: {}", Report::from_error(&e));
                    if fail_on_error {
                        return Err(e.into());
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
                        whatever!("{message}");
                    }
                    warn!(log: logger, "{}", message);
                }
                let user_path = format!("files/{username}/{path}");
                if let Err(e) = client
                    .delete_share(ShareId::from(shared_folder.read_share_id.clone()))
                    .await
                {
                    let message = format!(
                        "Could not delete NextCloud read share: {}",
                        Report::from_error(e)
                    );
                    if fail_on_error {
                        whatever!("{message}");
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
                    let message = format!(
                        "Could not delete NextCloud write share: {}",
                        Report::from_error(e)
                    );
                    if fail_on_error {
                        whatever!("{message}");
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
                    Err(opentalk_nextcloud_client::Error::FileNotFound { file_path, .. }) => {
                        debug!(
                            log: logger,
                            "Path {file_path:?} not found on the NextCloud instance, skipping deletion"
                        );
                    }
                    Err(e) => {
                        let message = format!(
                            "Error deleting folder on NextCloud: {}",
                            Report::from_error(e)
                        );
                        if fail_on_error {
                            whatever!("{message}");
                        }
                        warn!(log: logger, "{}", message);
                    }
                };
            }
            Ok(())
        }
    }
}
