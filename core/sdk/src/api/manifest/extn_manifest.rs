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
use std::{fs, path::Path};

use log::{info, warn};
use serde::Deserialize;

use crate::{extn::extn_id::ExtnId, utils::error::RippleError};

/// Contains the default path for the manifest
/// file extension type based on platform
#[derive(Deserialize, Debug, Clone)]
pub struct ExtnManifest {
    pub default_path: String,
    pub default_extension: String,
    pub extns: Vec<ExtnManifestEntry>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ExtnResolutionEntry {
    pub capability: String,
    pub priority: Option<u64>,
    pub exclusion: Option<bool>,
}

/// Contains Resolution strategies and path for the manifest.
#[derive(Deserialize, Debug, Clone)]
pub struct ExtnManifestEntry {
    pub path: String,
    pub symbols: Vec<ExtnSymbol>,
    pub resolution: Option<Vec<ExtnResolutionEntry>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ExtnSymbol {
    pub id: String,
    pub uses: Vec<String>,
    pub fulfills: Vec<String>,
}

impl ExtnSymbol {
    fn get_launcher_capability(&self) -> Option<ExtnId> {
        if let Ok(cap) = ExtnId::try_from(self.id.clone()) {
            if cap.is_launcher_channel() {
                return Some(cap);
            }
        }
        None
    }
}

impl ExtnManifestEntry {
    pub fn get_path(&self, default_path: &str, default_extn: &str) -> String {
        let path = self.path.clone();
        // has absolute path
        let path = match path.starts_with("/") {
            true => path,
            false => format!("{}{}", default_path, path),
        };

        let path = match Path::new(&path).extension() {
            Some(_) => path,
            None => format!("{}.{}", path, default_extn),
        };

        path
    }
}

impl ExtnManifest {
    pub fn load(path: String) -> Result<(String, ExtnManifest), RippleError> {
        info!("Trying to load device manifest from path={}", path);
        if let Ok(contents) = fs::read_to_string(&path) {
            Self::load_from_content(contents)
        } else {
            info!("No device manifest found in {}", path);
            Err(RippleError::MissingInput)
        }
    }

    pub fn load_from_content(contents: String) -> Result<(String, ExtnManifest), RippleError> {
        match serde_json::from_str::<ExtnManifest>(&contents) {
            Ok(manifest) => Ok((String::from(contents), manifest)),
            Err(err) => {
                warn!("{:?} could not load device manifest", err);
                Err(RippleError::InvalidInput)
            }
        }
    }

    pub fn get_launcher_capability(&self) -> Option<ExtnId> {
        for extn in self.extns.clone() {
            for symbol in extn.symbols {
                if let Some(cap) = symbol.get_launcher_capability() {
                    return Some(cap);
                }
            }
        }
        return None;
    }
}