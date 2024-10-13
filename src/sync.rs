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

use crate::models::{fur_shortcut::FurShortcut, fur_task::FurTask};

use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct SyncRequest {
    client_id: String,
    last_sync: i64,
    tasks: Vec<FurTask>,
    shortcuts: Vec<FurShortcut>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncResponse {
    pub server_timestamp: i64,
    pub tasks: Vec<FurTask>,
    pub shortcuts: Vec<FurShortcut>,
}

pub async fn sync_with_server(
    client_id: &str,
    last_sync: i64,
    tasks: Vec<FurTask>,
    shortcuts: Vec<FurShortcut>,
) -> Result<SyncResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let sync_request = SyncRequest {
        client_id: client_id.to_string(),
        last_sync,
        tasks,
        shortcuts,
    };

    let response = client
        .post("http://localhost:8662/sync") // TODO: Allow user to change server
        .json(&sync_request)
        .send()
        .await?
        .json::<SyncResponse>()
        .await?;

    Ok(response)
}
