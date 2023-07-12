use serde::Serialize;

#[derive(Serialize)]
pub struct PinResponse {
    pub status: String,
}
