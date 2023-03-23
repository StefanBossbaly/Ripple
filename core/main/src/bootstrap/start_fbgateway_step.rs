// If not stated otherwise in this file or this component's license file the
// following copyright and licenses apply:
//
// Copyright 2023 RDK Management
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

use crate::{
    firebolt::{
        firebolt_gateway::FireboltGateway,
        handlers::{
            device_rpc::DeviceRPCProvider, lcm_rpc::LifecycleManagementProvider,
            lifecycle_rpc::LifecycleRippleProvider,
        },
        rpc::RippleRPCProvider,
    },
    processor::rpc_gateway_processor::RpcGatewayProcessor,
    state::{bootstrap_state::BootstrapState, platform_state::PlatformState},
};
use jsonrpsee::core::{async_trait, server::rpc_module::Methods};
use ripple_sdk::{framework::bootstrap::Bootstep, utils::error::RippleError};

pub struct FireboltGatewayStep;

impl FireboltGatewayStep {
    async fn init_handlers(&self, state: PlatformState, extn_methods: Option<Methods>) -> Methods {
        let mut methods = Methods::new();
        let _ = methods.merge(DeviceRPCProvider::provide(state.clone()));
        let _ = methods.merge(LifecycleRippleProvider::provide(state.clone()));
        // LCM Api(s) not required for internal launcher
        if !state.has_internal_launcher() {
            let _ = methods.merge(LifecycleManagementProvider::provide(state.clone()));
        }
        if extn_methods.is_some() {
            let _ = methods.merge(extn_methods.unwrap());
        }

        methods
    }
}

#[async_trait]
impl Bootstep<BootstrapState> for FireboltGatewayStep {
    fn get_name(&self) -> String {
        "FireboltGatewayStep".into()
    }

    async fn setup(&self, state: BootstrapState) -> Result<(), RippleError> {
        let methods = self.init_handlers(state.platform_state.clone(), None).await;
        let gateway = FireboltGateway::new(state.clone(), methods);
        // Main can now recieve RPC requests
        state
            .platform_state
            .get_client()
            .add_request_processor(RpcGatewayProcessor::new(state.platform_state.get_client()));
        gateway.start().await;
        Ok(())
    }
}