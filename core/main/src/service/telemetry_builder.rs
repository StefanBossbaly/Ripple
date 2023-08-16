// Copyright 2023 Comcast Cable Communications Management, LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0
//

use ripple_sdk::{
    api::{
        firebolt::{
            fb_metrics::{ErrorParams, InternalInitializeParams},
            fb_telemetry::{
                AppLoadStart, AppLoadStop, FireboltInteraction, InternalInitialize,
                TelemetryAppError, TelemetryPayload, TelemetrySignIn, TelemetrySignOut,
            },
        },
        gateway::rpc_gateway_api::{CallContext, RpcRequest},
    },
    chrono::{DateTime, Utc},
    framework::RippleResponse,
    log::error,
};
use serde_json::Value;

use crate::state::platform_state::PlatformState;

pub struct TelemetryBuilder;
include!(concat!(env!("OUT_DIR"), "/version.rs"));

impl TelemetryBuilder {
    pub fn send_app_load_start(
        ps: &PlatformState,
        app_id: String,
        app_version: Option<String>,
        start_time: Option<DateTime<Utc>>,
    ) {
        if let Err(e) = Self::send_telemetry(
            ps,
            TelemetryPayload::AppLoadStart(AppLoadStart {
                app_id,
                app_version,
                start_time: start_time.unwrap_or_default().timestamp_millis(),
                ripple_session_id: ps.metrics.get_context().device_session_id,
                ripple_version: String::from(SEMVER),
                ripple_context: None,
            }),
        ) {
            error!("send_telemetry={:?}", e)
        }
    }

    pub fn send_app_load_stop(ps: &PlatformState, app_id: String, success: bool) {
        if let Err(e) = Self::send_telemetry(
            ps,
            TelemetryPayload::AppLoadStop(AppLoadStop {
                app_id,
                stop_time: Utc::now().timestamp_millis(),
                app_session_id: None,
                ripple_session_id: ps.metrics.get_context().device_session_id,
                success,
            }),
        ) {
            error!("send_telemetry={:?}", e)
        }
    }

    pub fn update_session_id_and_send_telemetry(
        ps: &PlatformState,
        mut t: TelemetryPayload,
    ) -> RippleResponse {
        let session_id = ps.metrics.get_context().device_session_id;
        t.update_session_id(session_id);
        Self::send_telemetry(ps, t)
    }

    fn send_telemetry(ps: &PlatformState, t: TelemetryPayload) -> RippleResponse {
        let listeners = ps.metrics.get_listeners();
        let client = ps.get_client().get_extn_client();
        let mut result = Ok(());
        for id in listeners {
            if let Err(e) = client.send_event_with_id(&id, t.clone()) {
                error!("telemetry_send_error target={} event={:?}", id, t.clone());
                result = Err(e)
            }
        }
        result
    }

    pub fn send_ripple_telemetry(ps: &PlatformState) {
        Self::send_app_load_start(
            ps,
            "ripple".to_string(),
            Some(String::from(SEMVER)),
            Some(ps.metrics.start_time),
        );
        Self::send_app_load_stop(ps, "ripple".to_string(), true);
    }

    pub fn send_error(ps: &PlatformState, app_id: String, error_params: ErrorParams) {
        let mut app_error: TelemetryAppError = error_params.into();
        app_error.ripple_session_id = ps.metrics.get_context().device_session_id;
        app_error.app_id = app_id;

        if let Err(e) = Self::send_telemetry(ps, TelemetryPayload::AppError(app_error)) {
            error!("send_telemetry={:?}", e)
        }
    }

    pub fn send_sign_in(ps: &PlatformState, ctx: &CallContext) {
        if let Err(e) = Self::send_telemetry(
            ps,
            TelemetryPayload::SignIn(TelemetrySignIn {
                app_id: ctx.app_id.to_owned(),
                ripple_session_id: ps.metrics.get_context().device_session_id,
                app_session_id: Some(ctx.session_id.to_owned()),
            }),
        ) {
            error!("send_telemetry={:?}", e)
        }
    }

    pub fn send_sign_out(ps: &PlatformState, ctx: &CallContext) {
        if let Err(e) = Self::send_telemetry(
            ps,
            TelemetryPayload::SignOut(TelemetrySignOut {
                app_id: ctx.app_id.to_owned(),
                ripple_session_id: ps.metrics.get_context().device_session_id,
                app_session_id: Some(ctx.session_id.to_owned()),
            }),
        ) {
            error!("send_telemetry={:?}", e)
        }
    }

    pub fn internal_initialize(
        ps: &PlatformState,
        ctx: &CallContext,
        params: &InternalInitializeParams,
    ) {
        if let Err(e) = Self::send_telemetry(
            ps,
            TelemetryPayload::InternalInitialize(InternalInitialize {
                app_id: ctx.app_id.to_owned(),
                ripple_session_id: ps.metrics.get_context().device_session_id,
                app_session_id: Some(ctx.session_id.to_owned()),
                semantic_version: params.value.to_string(),
            }),
        ) {
            error!("send_telemetry={:?}", e)
        }
    }

    pub fn send_fb_tt(ps: &PlatformState, req: RpcRequest, tt: i64, success: bool) {
        let ctx = req.ctx;
        let method = req.method;
        let params = if let Ok(mut p) = serde_json::from_str::<Vec<Value>>(&req.params_json) {
            if p.len() > 0 {
                // remove call context
                let _ = p.remove(0);
                Some(serde_json::to_string(&p).unwrap())
            } else {
                None
            }
        } else {
            None
        };
        if let Err(e) = Self::send_telemetry(
            ps,
            TelemetryPayload::FireboltInteraction(FireboltInteraction {
                app_id: ctx.app_id.to_owned(),
                ripple_session_id: ps.metrics.get_context().device_session_id,
                app_session_id: Some(ctx.session_id),
                tt,
                method,
                params,
                success,
            }),
        ) {
            error!("send_telemetry={:?}", e)
        }
    }
}