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

#[derive(Serialize, Deserialize)]
pub struct SyncRequest {
    last_sync: i64,
    tasks: Vec<EncryptedTask>,
    shortcuts: Vec<EncryptedShortcut>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncResponse {
    pub server_timestamp: i64,
    pub tasks: Vec<EncryptedTask>,
    pub shortcuts: Vec<EncryptedShortcut>,
}

#[derive(Deserialize)]
pub struct OrphanedItemsResponse {
    pub task_uids: Vec<String>,
    pub shortcut_uids: Vec<String>,
}

pub async fn sync_with_server(
    user: &FurUser,
    last_sync: i64,
    tasks: Vec<EncryptedTask>,
    shortcuts: Vec<EncryptedShortcut>,
) -> Result<SyncResponse, ApiError> {
    let client = Client::new();
    let sync_request = SyncRequest {
        last_sync,
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
        Err(ApiError::Server("Sync failed".into()))
    }
}

pub async fn fetch_orphaned_items(user: &FurUser) -> Result<OrphanedItemsResponse, ApiError> {
    let client = Client::new();
    let mut response = client
        .get(format!("{}/api/orphaned", user.server))
        .bearer_auth(&user.access_token)
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
            .get(format!("{}/api/orphaned", user.server))
            .bearer_auth(&user.access_token)
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
        Err(ApiError::Server("Sync failed".into()))
    }
}
