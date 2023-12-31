use crate::blaze::{
    components,
    models::auth::*,
    packet::Packet,
    router::{Blaze, SessionAuth},
    session::SessionLink,
};

pub async fn auth(
    session: SessionLink,
    SessionAuth(user): SessionAuth,
    Blaze(_req): Blaze<AuthRequest>,
) -> Blaze<AuthResponse> {
    let mut packet = Packet::notify(
        components::user_sessions::COMPONENT,
        components::user_sessions::UPDATE_AUTH,
        AuthNotify { user: user.clone() },
    );

    packet.frame.notify = 1;
    session.notify_handle().notify(packet);

    Blaze(AuthResponse { user })
}

#[rustfmt::skip]
static ENTITLEMENTS: &[Entitlement] = &[
    Entitlement::offer(1015257246559, "313772", 2, "Origin.OFR.50.0002307", "ME4_MP_BOOSTERPACK4", 5),
    Entitlement::offer(1015257046559, "313772", 2, "Origin.OFR.50.0002288", "ME4_MP_BOOSTERPACK1", 5),
    Entitlement::content(1015256846559, "313772", 2, "Origin.OFR.50.0001745", "ME4_PRO_PREORDER", 5),
    Entitlement::content(1015256646559, "313772", 2, "Origin.OFR.50.0001744", "ME4_MTX_DELUXE_ITEMS", 5),
    Entitlement::pc(1015256446559, "314574", 2, "Origin.OFR.50.0001649", "ONLINE_ACCESS", 1),
    Entitlement::content(1015256246559, "313772", 2, "Origin.OFR.50.0001744", "ME4_MTX_DELUXE_ITEMS", 5),
    Entitlement::content(1015256046559, "313772", 2, "Origin.OFR.50.0001745", "ME4_PRO_PREORDER", 5),
    Entitlement::content(1015255846559, "313772", 2, "Origin.OFR.50.0001746", "ME4_MTX_SOUNDTRACK", 5),
    Entitlement::offer(1015255646559, "313772", 2, "Origin.OFR.50.0002288", "ME4_MP_BOOSTERPACK1", 5),
    Entitlement::offer(1015255446559, "313772", 2, "Origin.OFR.50.0002307", "ME4_MP_BOOSTERPACK4", 5),
    Entitlement::offer(1014181546559, "313772", 2, "Origin.OFR.50.0002307", "ME4_MP_BOOSTERPACK4", 5),
    Entitlement::offer(1014181346559, "313772", 2, "Origin.OFR.50.0002288", "ME4_MP_BOOSTERPACK1", 5),
    Entitlement::content(1014181146559, "313772", 2, "Origin.OFR.50.0001746", "ME4_MTX_SOUNDTRACK", 5),
    Entitlement::content(1014180946559, "313772", 2, "Origin.OFR.50.0001745", "ME4_PRO_PREORDER", 5),
    Entitlement::content(1014180746559, "313772", 2, "Origin.OFR.50.0001744", "ME4_MTX_DELUXE_ITEMS", 5),
    Entitlement::pc(1014180546559, "314574", 2, "Origin.OFR.50.0001646", "ONLINE_ACCESS", 1),
    Entitlement::pc(1011177546559, "310335", 2, "Origin.OFR.50.0001530", "TRIAL_ONLINE_ACCESS", 1),
];

pub async fn list_entitlements_2() -> Blaze<ListEntitlementsResponse> {
    Blaze(ListEntitlementsResponse { list: ENTITLEMENTS })
}
