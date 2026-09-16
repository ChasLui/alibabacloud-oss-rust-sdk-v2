use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Default, Serialize, Clone)]
/// Allow for headers and parameters override, case-insensitive
pub struct RequestCommon {
    pub headers: HashMap<String, String>,
    pub parameters: HashMap<String, String>,
    pub payload: Option<Vec<u8>>,
}

#[derive(Debug, Default)]
pub struct ResultCommon {
    pub status: http::StatusCode,
    pub headers: HashMap<String, String>,
}
