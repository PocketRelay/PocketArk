use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct QosQuery {
    pub vers: u32,
    pub qtyp: u32,
    pub prpt: u16,
}

#[derive(Debug, Deserialize)]
pub struct FirewallQuery {
    pub vers: u32,
    pub nint: u32,
}
