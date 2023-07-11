use crate::blaze::pk::{codec::Encodable, tag::TdfType, writer::TdfWriter};

pub struct CreateGameResp;

impl Encodable for CreateGameResp {
    fn encode(&self, w: &mut TdfWriter) {
        w.tag_str_empty(b"COID");
        w.tag_str_empty(b"ESNM");
        w.tag_u32(b"MSID", 1);
        w.tag_str_empty(b"SCID");
        w.tag_str_empty(b"STMN");
    }
}

pub struct NotifyMatchmakingStatus;

impl Encodable for NotifyMatchmakingStatus {
    fn encode(&self, w: &mut TdfWriter) {
        {
            w.tag_list_start(b"ASIL", TdfType::Group, 1);

            w.group(b"CGS", |w| {
                w.tag_u8(b"EVST", 0);
                w.tag_u8(b"MMSN", 1);
                w.tag_u8(b"NOMP", 0);
            });

            w.group(b"FGS", |w| {
                w.tag_u8(b"GNUM", 0);
            });

            w.group(b"GEOS", |w| {
                w.tag_u8(b"DIST", 0);
            });
            w.group(b"HBRD", |w| {
                w.tag_u8(b"BVAL", 0);
            });
            w.group(b"HVRD", |w| {
                w.tag_u8(b"VVAL", 0);
            });
            w.group(b"PLCN", |w| {
                w.tag_u8(b"PMAX", 1);
                w.tag_u8(b"PMIN", 1);
            });
            w.group(b"PLUT", |w| {
                w.tag_u8(b"PMAX", 0);
                w.tag_u8(b"PMIN", 0);
            });
            w.group(b"PSRS", |_| {});
            w.group(b"RRDA", |w| {
                w.tag_u8(b"RVAL", 0);
            });
            w.group(b"TBRS", |w| {
                w.tag_u8(b"SDIF", 0);
            });
            w.group(b"TCPS", |w| {
                w.tag_str_empty(b"NAME");
            });
            w.group(b"TMSS", |w| {
                w.tag_u8(b"PCNT", 0);
            });
            w.group(b"TOTS", |w| {
                w.tag_u8(b"PMAX", 4);
                w.tag_u8(b"PMIN", 4);
            });
            w.group(b"TPPS", |w| {
                w.tag_u8(b"BDIF", 0);
                w.tag_u8(b"BOTN", 0);
                w.tag_str_empty(b"NAME");
                w.tag_u8(b"TDIF", 0);
                w.tag_u8(b"TOPN", 0);
            });
            w.group(b"TPPS", |w| {
                w.tag_u8(b"MUED", 0);
                w.tag_str_empty(b"NAME");
                w.tag_u8(b"SDIF", 0);
            });
            w.group(b"VGRS", |w| {
                w.tag_u8(b"VVAL", 0);
            });
            w.tag_group_end();
        }
        w.tag_u32(b"MSCD", 1); // pid
        w.tag_u32(b"MSID", 1); // pid
        w.tag_u32(b"USID", 1); // pid
    }
}
