use crate::{
    blaze::pk::{
        codec::{Decodable, Encodable, ValueType},
        error::DecodeResult,
        reader::TdfReader,
        tag::TdfType,
        writer::TdfWriter,
    },
    database::entity::User,
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

pub struct AuthNotify {
    pub user: User,
}

impl Encodable for AuthNotify {
    fn encode(&self, w: &mut TdfWriter) {
        w.tag_zero(b"\x17CON");
        w.tag_u32(b"ALOC", 1701727834); // location
        w.tag_u32(b"BUID", self.user.id);
        w.tag_triple(b"CGID", (30722u64, 2u64, self.user.id));
        w.tag_str(b"DSNM", &self.user.username);
        w.tag_zero(b"FRST");
        w.tag_str(b"KEY", "0");
        w.tag_u32(b"LAST", 1688871852);
        w.tag_u32(b"LLOG", 1688871991);
        w.tag_str(b"MAIL", "******@gmail.com");
        w.tag_str(b"NASP", "cem_ea_id");
        w.tag_u32(b"PID", self.user.id);
        w.tag_u8(b"PLAT", 4);
        w.tag_u32(b"UID", self.user.id);
        w.tag_u8(b"USTP", 0);
        w.tag_u32(b"XREF", self.user.id); //pid nucleus
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

pub struct Entitlement {
    pub name: &'static str,
    pub id: u64,
    pub pjid: &'static str,
    pub prca: u8,
    pub prid: &'static str,
    pub tag: &'static str,
    pub ty: u8,
}

impl Entitlement {
    pub const TAG_OFFER: &'static str = "ME4PCOffers";
    pub const TAG_CONTENT: &'static str = "ME4PCContent";
    pub const TAG_PC: &'static str = "ME4PC";

    pub const fn new_offer(
        id: u64,
        pjid: &'static str,
        prca: u8,
        prid: &'static str,
        tag: &'static str,
        ty: u8,
    ) -> Self {
        Self {
            name: Self::TAG_OFFER,
            id,
            pjid,
            prca,
            prid,
            tag,
            ty,
        }
    }

    pub const fn new_content(
        id: u64,
        pjid: &'static str,
        prca: u8,
        prid: &'static str,
        tag: &'static str,
        ty: u8,
    ) -> Self {
        Self {
            name: Self::TAG_CONTENT,
            id,
            pjid,
            prca,
            prid,
            tag,
            ty,
        }
    }
    pub const fn new_pc(
        id: u64,
        pjid: &'static str,
        prca: u8,
        prid: &'static str,
        tag: &'static str,
        ty: u8,
    ) -> Self {
        Self {
            name: Self::TAG_PC,
            id,
            pjid,
            prca,
            prid,
            tag,
            ty,
        }
    }
}

impl Encodable for Entitlement {
    fn encode(&self, writer: &mut TdfWriter) {
        writer.tag_str_empty(b"DEVI");
        writer.tag_str(b"GDAY", "2012-12-15T16:15Z");
        writer.tag_str(b"GNAM", self.name);
        writer.tag_u64(b"ID", self.id);
        writer.tag_u8(b"ISCO", 0);
        writer.tag_u8(b"PID", 0);
        writer.tag_str(b"PJID", self.pjid);
        writer.tag_u8(b"PRCA", self.prca);
        writer.tag_str(b"PRID", self.prid);
        writer.tag_u8(b"STAT", 1);
        writer.tag_u8(b"STRC", 0);
        writer.tag_str(b"TAG", self.tag);
        writer.tag_str_empty(b"TDAY");
        writer.tag_u8(b"TTYPE", self.ty);
        writer.tag_u8(b"UCNT", 0);
        writer.tag_u8(b"VER", 0);
        writer.tag_group_end();
    }
}

impl ValueType for Entitlement {
    fn value_type() -> TdfType {
        TdfType::Group
    }
}

pub struct ListEntitlementsResponse {
    pub list: &'static [Entitlement],
}

impl Encodable for ListEntitlementsResponse {
    fn encode(&self, writer: &mut TdfWriter) {
        writer.tag_slice_list(b"NLST", self.list);
    }
}
