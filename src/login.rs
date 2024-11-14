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

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::encryption::generate_device_id;

#[derive(Clone, Debug)]
pub enum ApiError {
    Network(Arc<reqwest::Error>),
    Auth(String),
    Server(String),
    TokenRefresh(String),
    Device(String),
}

#[derive(Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub encryption_key: String,
    pub device_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Serialize)]
struct RefreshRequest {
    refresh_token: String,
    device_id: String,
}

#[derive(Deserialize)]
struct RefreshResponse {
    access_token: String,
}

pub async fn login(
    email: String,
    encryption_key: String,
    server: String,
) -> Result<LoginResponse, ApiError> {
    let client = Client::new();
    let device_id = match generate_device_id() {
        Ok(id) => id,
        Err(_) => return Err(ApiError::Device("Failed to generate device ID".into())),
    };

    let response = client
        .post(format!("{}/api/login", server))
        .json(&LoginRequest {
            email,
            encryption_key,
            device_id,
        })
        .send()
        .await
        .map_err(|e| ApiError::Network(Arc::new(e)))?;

    if response.status().is_success() {
        response
            .json()
            .await
            .map_err(|e| ApiError::Network(Arc::new(e)))
    } else {
        Err(ApiError::Auth("Invalid credentials".into()))
    }
}

pub async fn refresh_auth_token(refresh_token: String, server: &str) -> Result<String, ApiError> {
    let client = Client::new();
    let device_id = match generate_device_id() {
        Ok(id) => id,
        Err(_) => return Err(ApiError::Device("Failed to generate device ID".into())),
    };

    let response = client
        .post(format!("{}/api/refresh", server))
        .json(&RefreshRequest {
            refresh_token,
            device_id,
        })
        .send()
        .await
        .map_err(|e| ApiError::Network(Arc::new(e)))?;

    if response.status().is_success() {
        let refresh_response = response
            .json::<RefreshResponse>()
            .await
            .map_err(|e| ApiError::Network(Arc::new(e)))?;
        Ok(refresh_response.access_token)
    } else {
        Err(ApiError::TokenRefresh("Failed to refresh token".into()))
    }
}
