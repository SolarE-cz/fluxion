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

use bevy_ecs::prelude::Resource;
use std::future::Future;

/// Resource that provides access to async task spawning
/// Uses tokio runtime which is required for reqwest HTTP client
#[derive(Resource, Clone)]
pub struct AsyncRuntime;

impl AsyncRuntime {
    /// Create a new AsyncRuntime
    pub fn new() -> Self {
        Self
    }

    /// Spawn an async task using tokio
    /// Returns a JoinHandle that can be detached
    pub fn spawn<T>(
        &self,
        future: impl Future<Output = T> + Send + 'static,
    ) -> tokio::task::JoinHandle<T>
    where
        T: Send + 'static,
    {
        tokio::spawn(future)
    }
}

impl Default for AsyncRuntime {
    fn default() -> Self {
        Self::new()
    }
}
