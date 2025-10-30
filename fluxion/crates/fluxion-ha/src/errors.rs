// Copyright (c) 2025 SOLARE S.R.O.
//
// This file is part of FluxION.
//
// FluxION is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// FluxION is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with FluxION. If not, see <https://www.gnu.org/licenses/>.
//
// For commercial licensing, please contact: info@solare.cz

use thiserror::Error;

/// Home Assistant API error types
#[derive(Error, Debug)]
pub enum HaError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("HA API returned error status {status}: {message}")]
    ApiError { status: u16, message: String },

    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Service call failed: {service} - {reason}")]
    ServiceCallFailed { service: String, reason: String },

    #[error("Connection timeout")]
    Timeout,

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type HaResult<T> = Result<T, HaError>;
