/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Wi-Fi information reader using the standard nl80211 driver.
//! Only works on Linux 2.6 and later.
//! Most reliable source of truth regarding Wi-Fi, due to inconsistencies in procfs and sysfs reporting.

use super::WifiInfo;
use anyhow::Context;
use neli::{
    consts::{
        nl::{NlTypeWrapper, NlmF},
        socket::NlFamily,
    },
    genl::{AttrTypeBuilder, Genlmsghdr, GenlmsghdrBuilder, NlattrBuilder},
    nl::NlPayload,
    router::synchronous::NlRouter,
    types::{Buffer, GenlBuffer},
    utils::Groups,
};

/// Wi-Fi reader convenience struct
pub struct WifiReader {
    router: neli::router::synchronous::NlRouter,
    nl80211: u16,
}

impl WifiReader {
    pub fn new() -> anyhow::Result<Self> {
        let (router, _) = NlRouter::connect(NlFamily::Generic, None, Groups::empty())?;

        // Send a message to the generic controller to get the nl80211 family ID
        let nl80211 = router
            .resolve_genl_family("nl80211")
            .context("nl80211 family not found — is cfg80211 loaded?")?;
        Ok(Self { router, nl80211 })
    }

    pub fn read(&mut self, iface: &str) -> anyhow::Result<WifiInfo> {
        let ifindex = std::fs::read_to_string(format!("/sys/class/net/{}/ifindex", iface))
            .context("failed to read ifindex for interface")?
            .trim()
            .parse::<u32>()
            .context("invalid ifindex for interface")?;

        let interface = self.read_interface(ifindex)?;
        let station = self.read_station(interface.index)?;
        Ok(WifiInfo {
            ssid: interface.ssid,
            signal_strength_dbm: station.signal_strength as i32,
            frequency_mhz: interface.frequency,
            link_speed_up_mbps: station.link_speed_up,
            link_speed_down_mbps: station.link_speed_down,
        })
    }

    /// Reads interface info from the nl80211 driver for the given interface index.
    /// Interface info regards values stored inside the network card, including information needed to communicate with the remote access point, such as SSID and frequency
    fn read_interface(&mut self, ifindex: u32) -> anyhow::Result<InterfaceInfo> {
        // Send a NL80211_CMD_GET_INTERFACE request to the nl80211 driver to get interface info for the given interface index
        let recv = self
            .router
            .send::<_, _, NlTypeWrapper, Genlmsghdr<Cmd, Attr>>(
                self.nl80211,
                NlmF::REQUEST | NlmF::ACK | NlmF::DUMP,
                NlPayload::Payload(
                    GenlmsghdrBuilder::<Cmd, Attr>::default()
                        .cmd(NL80211_CMD_GET_INTERFACE)
                        .version(1)
                        .attrs({
                            let mut attrs = GenlBuffer::new();
                            // Set the interface index attribute so the driver knows which interface to query
                            attrs.push(
                                NlattrBuilder::default()
                                    .nla_type(
                                        AttrTypeBuilder::default()
                                            .nla_type(NL80211_ATTR_IFINDEX)
                                            .build()?,
                                    )
                                    .nla_payload(ifindex)
                                    .build()?,
                            );

                            attrs
                        })
                        .build()?,
                ),
            )?;

        let mut interface: Option<InterfaceInfo> = None;

        // Iterate over response messages, looking for a payload with the needed attributes
        for msg in recv {
            let msg = msg?;
            if let NlPayload::Payload(payload) = msg.nl_payload() {
                for attr in payload.attrs().iter() {
                    let buffer = attr.nla_payload().to_owned();
                    let buf = buffer.as_ref();
                    // Check for interface type 2 (NL80211_IFTYPE_STATION) because that's the only type we're interested in
                    if *attr.nla_type().nla_type() == NL80211_ATTR_IFTYPE
                        && attr.nla_payload().as_ref()[0] != 2
                    {
                        continue;
                    }

                    match *attr.nla_type().nla_type() {
                        // The interface index, is supposed to match the ifindex but I'm not taking any chances
                        NL80211_ATTR_IFINDEX => {
                            interface.get_or_insert_default().index = buffer;
                        }
                        // The Wi-Fi frequency in MHz
                        NL80211_ATTR_WIPHY_FREQ => {
                            interface.get_or_insert_default().frequency =
                                u32::from_ne_bytes(buf[..4].try_into()?);
                        }
                        // The SSID of the Wi-Fi network, there's no standard about the encoding, it's just a raw byte sequence but I'm assuming UTF-8 due to prevalence
                        NL80211_ATTR_SSID => {
                            interface.get_or_insert_default().ssid =
                                String::from_utf8_lossy(buf).into_owned();
                        }
                        _ => {}
                    }
                }
            }
        }
        interface.ok_or_else(|| anyhow::anyhow!("Interface could not be found"))
    }

    /// Reads station info from the nl80211 driver for the given interface index.
    /// Station info regards values relating to the remote access point, including link speed and signal strength.
    fn read_station(&mut self, interface: Buffer) -> anyhow::Result<StationInfo> {
        // Send a NL80211_CMD_GET_STATION request to the nl80211 driver to get station info for the given interface
        let recv = self
            .router
            .send::<_, Genlmsghdr<Cmd, Attr>, NlTypeWrapper, Genlmsghdr<Cmd, Attr>>(
                self.nl80211,
                NlmF::REQUEST | NlmF::ACK | NlmF::DUMP,
                NlPayload::Payload(
                    GenlmsghdrBuilder::<Cmd, Attr>::default()
                        .cmd(NL80211_CMD_GET_STATION)
                        .version(NL80211_GENL_VERSION)
                        .attrs({
                            let mut attrs = GenlBuffer::new();
                            // Set the interface index attribute so the driver knows which interface to query
                            attrs.push(
                                NlattrBuilder::default()
                                    .nla_type(
                                        AttrTypeBuilder::default()
                                            .nla_type(NL80211_ATTR_IFINDEX)
                                            .build()?,
                                    )
                                    .nla_payload(interface)
                                    .build()?,
                            );
                            attrs
                        })
                        .build()?,
                ),
            )?;

        let mut station: Option<StationInfo> = None;

        // Iterate over response messages, looking for a payload with the needed attributes
        for msg in recv {
            let msg = msg?;
            if let NlPayload::Payload(payload) = msg.nl_payload() {
                for attr in payload.attrs().iter() {
                    // Look for the NL80211_ATTR_STA_INFO attribute, which contains station information in nested format
                    if *attr.nla_type().nla_type() == NL80211_ATTR_STA_INFO
                        && let Ok(handle) = attr.get_attr_handle::<Attr>()
                    {
                        // Iterate over the nested attributes to find the data needed
                        for sta_attr in handle.iter() {
                            let buffer = sta_attr.nla_payload();
                            let buf = buffer.as_ref();
                            match *sta_attr.nla_type().nla_type() {
                                // The NL80211_STA_INFO_TX_BITRATE attribute contains the station's link speed
                                NL80211_STA_INFO_TX_BITRATE => {
                                    if let Ok(nest_handle) = sta_attr.get_attr_handle::<Attr>() {
                                        for rate_attr in nest_handle.iter() {
                                            let buffer = rate_attr.nla_payload();
                                            let buf = buffer.as_ref();
                                            if *rate_attr.nla_type().nla_type()
                                                == NL80211_RATE_INFO_BITRATE32
                                            {
                                                station.get_or_insert_default().link_speed_up =
                                                    u32::from_ne_bytes(buf[..4].try_into()?);
                                            }
                                        }
                                    }
                                }
                                NL80211_STA_INFO_RX_BITRATE => {
                                    if let Ok(nest_handle) = sta_attr.get_attr_handle::<Attr>() {
                                        for rate_attr in nest_handle.iter() {
                                            let buffer = rate_attr.nla_payload();
                                            let buf = buffer.as_ref();
                                            if *rate_attr.nla_type().nla_type()
                                                == NL80211_RATE_INFO_BITRATE32
                                            {
                                                station.get_or_insert_default().link_speed_down =
                                                    u32::from_ne_bytes(buf[..4].try_into()?);
                                            }
                                        }
                                    }
                                }
                                NL80211_STA_INFO_SIGNAL_AVG => {
                                    station
                                        .get_or_insert(StationInfo {
                                            link_speed_up: 0,
                                            link_speed_down: 0,
                                            signal_strength: 0,
                                        })
                                        .signal_strength = buf[0] as i8;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        station.ok_or_else(|| anyhow::anyhow!("No station info found"))
    }
}

/// Information retrieved from the nl80211 driver for a given interface.
#[derive(Default)]
struct InterfaceInfo {
    ssid: String,
    frequency: u32,
    index: Buffer,
}

/// Information retrieved from the nl80211 driver for a given station (remote access point).
#[derive(Default)]
struct StationInfo {
    link_speed_up: u32,
    link_speed_down: u32,
    signal_strength: i8,
}

// Command and attribute identifier data types as defined in the NetLink protocol specs. Command identifiers are unsigned 8-bit integers, attribute identifiers are unsigned 16-bit integers.
type Cmd = u8;
type Attr = u16;

// NL80211 constants, gathered from linux/include/uapi/linux/nl80211.h enums
const NL80211_GENL_VERSION: u8 = 1;
const NL80211_CMD_GET_INTERFACE: u8 = 5;
const NL80211_CMD_GET_STATION: u8 = 17;

const NL80211_ATTR_IFINDEX: u16 = 3;
const NL80211_ATTR_IFTYPE: u16 = 5;
const NL80211_ATTR_STA_INFO: u16 = 21;
const NL80211_ATTR_WIPHY_FREQ: u16 = 38;
const NL80211_ATTR_SSID: u16 = 52;

const NL80211_STA_INFO_TX_BITRATE: u16 = 8;
const NL80211_STA_INFO_SIGNAL_AVG: u16 = 13;
const NL80211_STA_INFO_RX_BITRATE: u16 = 14;

const NL80211_RATE_INFO_BITRATE32: u16 = 5;
