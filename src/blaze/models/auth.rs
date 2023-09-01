use tdf::{ObjectId, TdfDeserialize, TdfSerialize, TdfType, TdfTyped};

use crate::{blaze::components::user_sessions::PLAYER_SESSION_TYPE, database::entity::User};

#[derive(Debug, TdfDeserialize)]
pub struct AuthRequest {
    #[tdf(tag = "AUTH")]
    pub token: String,
}

pub struct AuthNotify {
    pub user: User,
}

impl TdfSerialize for AuthNotify {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        w.tag_zero(b"7CON");
        w.tag_u32(b"ALOC", 1701727834); // location
        w.tag_u32(b"BUID", self.user.id);

        w.tag_alt(
            b"CGID",
            ObjectId::new(PLAYER_SESSION_TYPE, self.user.id as u64),
        );

        w.tag_str(b"DSNM", &self.user.username);
        w.tag_zero(b"FRST");
        w.tag_str(b"KEY", "0");
        w.tag_u32(b"LAST", 1688871852); // Last login time
        w.tag_u32(b"LLOG", 1688871991); // Login time
        w.tag_str(b"MAIL", "******@gmail.com");
        w.tag_str(b"NASP", "cem_ea_id");
        w.tag_owned(b"PID", self.user.id);
        w.tag_u8(b"PLAT", 4);
        w.tag_owned(b"UID", self.user.id);
        w.tag_u8(b"USTP", 0);
        w.tag_owned(b"XREF", self.user.id); //pid nucleus
    }
}

pub struct AuthResponse {
    pub user: User,
}

impl TdfSerialize for AuthResponse {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        w.group(b"SESS", |w| {
            w.tag_zero(b"7CON");
            w.tag_u32(b"BUID", self.user.id);
            w.tag_zero(b"FRST");
            w.tag_str(b"KEY", "0");
            w.tag_u32(b"LLOG", 1688871991);
            w.tag_str(b"MAIL", "******@gmail.com");
            w.group(b"PDTL", |w| {
                w.tag_str(b"DSNM", &self.user.username);
                w.tag_u32(b"LAST", 0);
                w.tag_u32(b"PID", self.user.id);
                w.tag_u8(b"PLAT", 4);
                w.tag_u8(b"STAS", 0);
                w.tag_u32(b"XREF", self.user.id);
            });
            w.tag_u32(b"UID", self.user.id);
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

impl TdfSerialize for Entitlement {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        w.tag_str_empty(b"DEVI");
        w.tag_str(b"GDAY", "2012-12-15T16:15Z");
        w.tag_str(b"GNAM", self.name);
        w.tag_u64(b"ID", self.id);
        w.tag_u8(b"ISCO", 0);
        w.tag_u8(b"PID", 0);
        w.tag_str(b"PJID", self.pjid);
        w.tag_u8(b"PRCA", self.prca);
        w.tag_str(b"PRID", self.prid);
        w.tag_u8(b"STAT", 1);
        w.tag_u8(b"STRC", 0);
        w.tag_str(b"TAG", self.tag);
        w.tag_str_empty(b"TDAY");
        w.tag_u8(b"TTYPE", self.ty);
        w.tag_u8(b"UCNT", 0);
        w.tag_u8(b"VER", 0);
        w.tag_group_end();
    }
}

impl TdfTyped for Entitlement {
    const TYPE: TdfType = TdfType::Group;
}

#[derive(TdfSerialize)]
pub struct ListEntitlementsResponse {
    #[tdf(tag = "NLST")]
    pub list: &'static [Entitlement],
}
