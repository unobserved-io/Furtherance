// Furtherance - Track your time without being tracked
// Copyright (C) 2024  Ricky Kresslein <rk@unobserved.io>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::sync::Arc;

use crate::{
    database::db_update_access_token,
    models::{fur_shortcut::EncryptedShortcut, fur_task::EncryptedTask, fur_user::FurUser},
    server::login::{refresh_auth_token, ApiError},
};

use reqwest::{self, Client};
use serde::{Deserialize, Serialize};

use super::encryption;

#[derive(Serialize, Deserialize)]
pub struct SyncRequest {
    last_sync: i64,
    device_id: String,
    tasks: Vec<EncryptedTask>,
    shortcuts: Vec<EncryptedShortcut>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncResponse {
    pub server_timestamp: i64,
    pub tasks: Vec<EncryptedTask>,
    pub shortcuts: Vec<EncryptedShortcut>,
    pub orphaned_tasks: Vec<String>,
    pub orphaned_shortcuts: Vec<String>,
}

pub async fn sync_with_server(
    user: &FurUser,
    last_sync: i64,
    tasks: Vec<EncryptedTask>,
    shortcuts: Vec<EncryptedShortcut>,
) -> Result<SyncResponse, ApiError> {
    let client = Client::new();
    let device_id = encryption::generate_device_id().map_err(|e| {
        eprintln!("Failed to create device id for logout: {:?}", e);
        ApiError::Device("Failed to generate device ID".to_string())
    })?;

    let sync_request = SyncRequest {
        last_sync,
        device_id,
        tasks,
        shortcuts,
    };

    let mut response = client
        .post(format!("{}/api/sync", user.server))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .json(&sync_request)
        .send()
        .await
        .map_err(|e| ApiError::Network(Arc::new(e)))?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        // Try token refresh
        let new_access_token =
            refresh_auth_token(user.refresh_token.to_string(), &user.server).await?;
        if let Err(e) = db_update_access_token(&user.email, &new_access_token) {
            return Err(ApiError::TokenRefresh(e.to_string()));
        }

        // Retry with new token
        response = client
            .post(format!("{}/api/sync", user.server))
            .header("Authorization", format!("Bearer {}", new_access_token))
            .json(&sync_request)
            .send()
            .await
            .map_err(|e| ApiError::Network(Arc::new(e)))?;
    }

    if response.status().is_success() {
        response
            .json()
            .await
            .map_err(|e| ApiError::Network(Arc::new(e)))
    } else {
        if let Ok(error) = response.json::<serde_json::Value>().await {
            if let Some(error_type) = error.get("error").and_then(|e| e.as_str()) {
                if error_type == "inactive_subscription" {
                    return Err(ApiError::InactiveSubscription(
                        error
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Subscription inactive")
                            .to_string(),
                    ));
                }
            }
        }
        Err(ApiError::Server("Sync failed".into()))
    }
}
