use blaze_pk::{
    codec::{Decodable, Encodable},
    error::DecodeResult,
    reader::TdfReader,
    writer::TdfWriter,
};

pub struct AuthRequest {
    pub token: String,
}

impl Decodable for AuthRequest {
    fn decode(reader: &mut TdfReader) -> DecodeResult<Self> {
        let token = reader.tag(b"AUTH")?;
        Ok(Self { token })
    }
}

pub struct AuthNotify;

impl Encodable for AuthNotify {
    fn encode(&self, w: &mut TdfWriter) {
        w.tag_zero(b"\x17CON");
        w.tag_u32(b"ALOC", 1701727834);
        w.tag_u32(b"BUID", 978651371);
        w.tag_triple(b"CGID", (30722u64, 2u64, 1052287650009u64));
        w.tag_str(b"DSNM", "Jacobtread");
        w.tag_zero(b"FRST");
        w.tag_str(b"KEY", "0");
        w.tag_u32(b"LAST", 1688871852);
        w.tag_u32(b"LLOG", 1688871991);
        w.tag_str(b"MAIL", "******@gmail.com");
        w.tag_str(b"NASP", "cem_ea_id");
        w.tag_u32(b"PID", 978651371);
        w.tag_u8(b"PLAT", 4);
        w.tag_u64(b"UID", 1000279946559);
        w.tag_u8(b"USTP", 0);
        w.tag_u64(b"XREF", 1000279946559);
    }
}

pub struct AuthResponse;

impl Encodable for AuthResponse {
    fn encode(&self, w: &mut TdfWriter) {
        w.group(b"SESS", |w| {
            w.tag_zero(b"\x17CON");
            w.tag_u32(b"BUID", 978651371);
            w.tag_zero(b"FRST");
            w.tag_str(b"KEY", "0");
            w.tag_u32(b"LLOG", 1688871991);
            w.tag_str(b"MAIL", "******@gmail.com");
            w.group(b"PDTL", |w| {
                w.tag_str(b"DSNM", "Jacobtread");
                w.tag_u32(b"LAST", 0);
                w.tag_u32(b"PID", 978651371);
                w.tag_u8(b"PLAT", 4);
                w.tag_u8(b"STAS", 0);
                w.tag_u64(b"XREF", 1000279946559);
            });
            w.tag_u64(b"UID", 1000279946559);
        });
        w.tag_u8(b"SPAM", 0);
        w.tag_u8(b"UNDR", 0);
    }
}
