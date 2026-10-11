//! # Virtual Peering
//!
//! Unfortunately the [Dedicated Topology](https://jacobtread.com/blog/pocket-relay-tunnel/) tunneled networking
//! method does not work for Mass Effect Andromeda attempting to use that topology causes the game client to crash
//!
//! So instead of trying to force everyone to connect to a dedicated virtual peer we can use a virtual addressing
//! system thats intercepted by the local client to re-route the traffic to the tunneling socket system using the slot
//! encoded into the IP address

use std::net::Ipv4Addr;

use crate::blaze::models::user_sessions::{IpPairAddress, NetworkAddress, PairAddress};

const VIRTUAL_PEER_PORT: u16 = 7337;
const VIRTUAL_PEER_ADDRESS_OFFSET: u8 = 24;

/// Get a virtual peer address for a specific slot
pub fn virtual_peer_address(slot: u8) -> NetworkAddress {
    NetworkAddress::AddressPair(virtual_peer_address_pair(slot))
}
/// Get a virtual peer address for a specific slot
pub fn virtual_peer_address_pair(slot: u8) -> IpPairAddress {
    let suffix = VIRTUAL_PEER_ADDRESS_OFFSET
        .checked_add(slot)
        .expect("tried to create a virtual peer address outside of the u8 bounds");

    let maci = 0;
    let address = PairAddress {
        addr: Ipv4Addr::new(127, 0, 0, suffix),
        port: VIRTUAL_PEER_PORT,
        maci,
    };

    IpPairAddress {
        external: address.clone(),
        internal: address,
        maci,
    }
}
