use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct QosQuery {
    pub vers: u32,
    pub qtyp: u32,
    pub prpt: u16,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct FirewallQuery {
    pub vers: u32,
    pub nint: u32,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct FireTypeQuery {
    pub vers: u32,
    pub rqid: u32,
    pub rqsc: u32,
    pub inip: i64,
    pub inpt: u16,
}
