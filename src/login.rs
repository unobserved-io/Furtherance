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

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LoginResponse {
    pub password_hash: String,
    pub salt: String,
}

pub async fn login(
    email: String,
    password: String,
    server: String,
) -> Result<LoginResponse, String> {
    let client = reqwest::Client::new();

    let login_request = LoginRequest { email, password };

    let response = client
        .post(format!("{}/api/login", server))
        .json(&login_request)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        response
            .json::<LoginResponse>()
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Invalid credentials".to_string())
    }
}
