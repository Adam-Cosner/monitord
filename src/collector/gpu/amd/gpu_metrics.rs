/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Translated metrics from the gpu_metrics file in a Rust-friendly format.
use num::traits::AsPrimitive;
use std::path::Path;

/// Handler for the gpu_metrics file.
pub enum GpuMetrics {
    F1C0(amdgpu::f1::C0),
    F1C1(amdgpu::f1::C1),
    F1C2(amdgpu::f1::C2),
    F1C3(amdgpu::f1::C3),
    F1C4(amdgpu::f1::C4),
    F1C5(amdgpu::f1::C5),
    F1C6(amdgpu::f1::C6),
    F1C7(amdgpu::f1::C7),
    F1C8(amdgpu::f1::C8),
    F1C9(amdgpu::f1::C9),
    F2C0(amdgpu::f2::C0),
    F2C1(amdgpu::f2::C1),
    F2C2(amdgpu::f2::C2),
    F2C3(amdgpu::f2::C3),
    F2C4(amdgpu::f2::C4),
    F3C0(amdgpu::f3::C0),
}

impl GpuMetrics {
    pub fn read(path: &Path) -> anyhow::Result<Self> {
        let content_bytes = std::fs::read(path)?;
        let header = unsafe { &*(content_bytes.as_ptr() as *const amdgpu::Header) };
        match header.format_revision {
            1 => match header.content_revision {
                0 => amdgpu::f1::C0::from_bytes(&content_bytes),
                1 => amdgpu::f1::C1::from_bytes(&content_bytes),
                2 => amdgpu::f1::C2::from_bytes(&content_bytes),
                3 => amdgpu::f1::C3::from_bytes(&content_bytes),
                4 => amdgpu::f1::C4::from_bytes(&content_bytes),
                5 => amdgpu::f1::C5::from_bytes(&content_bytes),
                6 => amdgpu::f1::C6::from_bytes(&content_bytes),
                7 => amdgpu::f1::C7::from_bytes(&content_bytes),
                8 => amdgpu::f1::C8::from_bytes(&content_bytes),
                9 => amdgpu::f1::C9::from_bytes(&content_bytes),
                _ => anyhow::bail!("unsupported content revision"),
            },
            2 => match header.content_revision {
                0 => amdgpu::f2::C0::from_bytes(&content_bytes),
                1 => amdgpu::f2::C1::from_bytes(&content_bytes),
                2 => amdgpu::f2::C2::from_bytes(&content_bytes),
                3 => amdgpu::f2::C3::from_bytes(&content_bytes),
                4 => amdgpu::f2::C4::from_bytes(&content_bytes),
                _ => anyhow::bail!("unsupported content revision"),
            },
            3 => amdgpu::f3::C0::from_bytes(&content_bytes),
            _ => anyhow::bail!("unsupported format revision"),
        }
    }

    pub fn graphics_utilization(&self) -> f64 {
        use amdgpu::f1::AttrId;
        match self {
            GpuMetrics::F1C0(metrics) => metrics.average_gfx_activity as f64,
            GpuMetrics::F1C1(metrics) => metrics.average_gfx_activity as f64,
            GpuMetrics::F1C2(metrics) => metrics.average_gfx_activity as f64,
            GpuMetrics::F1C3(metrics) => metrics.average_gfx_activity as f64,
            GpuMetrics::F1C4(metrics) => metrics.average_gfx_activity as f64,
            GpuMetrics::F1C5(metrics) => metrics.average_gfx_activity as f64,
            GpuMetrics::F1C6(metrics) => metrics.average_gfx_activity as f64,
            GpuMetrics::F1C7(metrics) => metrics.average_gfx_activity as f64,
            GpuMetrics::F1C8(metrics) => metrics.average_gfx_activity as f64,
            GpuMetrics::F1C9(metrics) => average_without_sentinel(
                &metrics
                    .attributes
                    .iter()
                    .find(|attr| attr.id == AttrId::AverageGfxActivity)
                    .map(|gfx_activity| gfx_activity.get_as::<u16>())
                    .unwrap_or_default(),
            ) as f64,
            GpuMetrics::F2C0(metrics) => metrics.average_gfx_activity as f64,
            GpuMetrics::F2C1(metrics) => metrics.average_gfx_activity as f64,
            GpuMetrics::F2C2(metrics) => metrics.average_gfx_activity as f64,
            GpuMetrics::F2C3(metrics) => metrics.average_gfx_activity as f64,
            GpuMetrics::F2C4(metrics) => metrics.average_gfx_activity as f64,
            GpuMetrics::F3C0(metrics) => metrics.average_gfx_activity as f64,
        }
    }

    pub fn graphics_clock(&self) -> u32 {
        use amdgpu::f1::AttrId;
        match self {
            GpuMetrics::F1C0(metrics) => metrics.average_gfxclk_frequency as u32,
            GpuMetrics::F1C1(metrics) => metrics.average_gfxclk_frequency as u32,
            GpuMetrics::F1C2(metrics) => metrics.average_gfxclk_frequency as u32,
            GpuMetrics::F1C3(metrics) => metrics.average_gfxclk_frequency as u32,
            GpuMetrics::F1C4(metrics) => metrics
                .current_gfxclk
                .iter()
                .cloned()
                .filter(|clk| *clk != 0xFFFF)
                .sum::<u16>() as u32,
            GpuMetrics::F1C5(metrics) => metrics
                .current_gfxclk
                .iter()
                .cloned()
                .filter(|clk| *clk != 0xFFFF)
                .sum::<u16>() as u32,
            GpuMetrics::F1C6(metrics) => metrics
                .current_gfxclk
                .iter()
                .cloned()
                .filter(|clk| *clk != 0xFFFF)
                .sum::<u16>() as u32,
            GpuMetrics::F1C7(metrics) => metrics
                .current_gfxclk
                .iter()
                .cloned()
                .filter(|clk| *clk != 0xFFFF)
                .sum::<u16>() as u32,
            GpuMetrics::F1C8(metrics) => metrics
                .current_gfxclk
                .iter()
                .cloned()
                .filter(|clk| *clk != 0xFFFF)
                .sum::<u16>() as u32,
            GpuMetrics::F1C9(metrics) => average_without_sentinel(
                &metrics
                    .attributes
                    .iter()
                    .find(|attr| attr.id == AttrId::CurrentGfxclk)
                    .map(|gfx_activity| gfx_activity.get_as::<u16>())
                    .unwrap_or_default(),
            ) as u32,
            GpuMetrics::F2C0(metrics) => metrics.average_gfxclk_frequency as u32,
            GpuMetrics::F2C1(metrics) => metrics.average_gfxclk_frequency as u32,
            GpuMetrics::F2C2(metrics) => metrics.average_gfxclk_frequency as u32,
            GpuMetrics::F2C3(metrics) => metrics.average_gfxclk_frequency as u32,
            GpuMetrics::F2C4(metrics) => metrics.average_gfxclk_frequency as u32,
            GpuMetrics::F3C0(metrics) => metrics.average_gfxclk_frequency as u32,
        }
    }

    pub fn memory_clock(&self) -> u32 {
        use amdgpu::f1::AttrId;
        match self {
            GpuMetrics::F1C0(metrics) => metrics.average_uclk_frequency as u32,
            GpuMetrics::F1C1(metrics) => metrics.average_uclk_frequency as u32,
            GpuMetrics::F1C2(metrics) => metrics.average_uclk_frequency as u32,
            GpuMetrics::F1C3(metrics) => metrics.average_uclk_frequency as u32,
            GpuMetrics::F1C4(metrics) => metrics.current_uclk as u32,
            GpuMetrics::F1C5(metrics) => metrics.current_uclk as u32,
            GpuMetrics::F1C6(metrics) => metrics.current_uclk as u32,
            GpuMetrics::F1C7(metrics) => metrics.current_uclk as u32,
            GpuMetrics::F1C8(metrics) => metrics.current_uclk as u32,
            GpuMetrics::F1C9(metrics) => average_without_sentinel(
                &metrics
                    .attributes
                    .iter()
                    .find(|attr| attr.id == AttrId::CurrentUclk)
                    .map(|val| val.get_as::<u16>())
                    .unwrap_or_default(),
            ) as u32,
            GpuMetrics::F2C0(metrics) => metrics.average_uclk_frequency as u32,
            GpuMetrics::F2C1(metrics) => metrics.average_uclk_frequency as u32,
            GpuMetrics::F2C2(metrics) => metrics.average_uclk_frequency as u32,
            GpuMetrics::F2C3(metrics) => metrics.average_uclk_frequency as u32,
            GpuMetrics::F2C4(metrics) => metrics.average_uclk_frequency as u32,
            GpuMetrics::F3C0(metrics) => metrics.average_uclk_frequency as u32,
        }
    }

    // This one's fun cause they kept freakin CHANGING WHERE THEY STORED IT
    pub fn video_enc_dec_util(&self) -> f64 {
        use amdgpu::f1::AttrId;
        match self {
            GpuMetrics::F1C0(metrics) => metrics.average_mm_activity as f64,
            GpuMetrics::F1C1(metrics) => metrics.average_mm_activity as f64,
            GpuMetrics::F1C2(metrics) => metrics.average_mm_activity as f64,
            GpuMetrics::F1C3(metrics) => metrics.average_mm_activity as f64,
            GpuMetrics::F1C4(metrics) => average_without_sentinel(&metrics.vcn_activity) as f64,
            GpuMetrics::F1C5(metrics) => average_without_sentinel(&metrics.vcn_activity) as f64,
            GpuMetrics::F1C6(metrics) => average_without_sentinel(
                &metrics
                    .xcp_stats
                    .iter()
                    .map(|xcp| xcp.vcn_busy)
                    .flatten()
                    .collect::<Vec<u16>>(),
            ) as f64,
            GpuMetrics::F1C7(metrics) => average_without_sentinel(
                &metrics
                    .xcp_stats
                    .iter()
                    .map(|xcp| xcp.vcn_busy)
                    .flatten()
                    .collect::<Vec<u16>>(),
            ) as f64,
            GpuMetrics::F1C8(metrics) => average_without_sentinel(
                &metrics
                    .xcp_stats
                    .iter()
                    .map(|xcp| xcp.vcn_busy)
                    .flatten()
                    .collect::<Vec<u16>>(),
            ) as f64,
            GpuMetrics::F1C9(metrics) => average_without_sentinel(
                &metrics
                    .attributes
                    .iter()
                    .find(|attr| attr.id == AttrId::VcnBusy)
                    .map(|val| val.get_as::<u16>())
                    .unwrap_or_default(),
            ) as f64,
            GpuMetrics::F2C0(metrics) => metrics.average_mm_activity as f64,
            GpuMetrics::F2C1(metrics) => metrics.average_mm_activity as f64,
            GpuMetrics::F2C2(metrics) => metrics.average_mm_activity as f64,
            GpuMetrics::F2C3(metrics) => metrics.average_mm_activity as f64,
            GpuMetrics::F2C4(metrics) => metrics.average_mm_activity as f64,
            GpuMetrics::F3C0(metrics) => metrics.average_vcn_activity as f64,
        }
    }

    pub fn encoder_clock(&self) -> u32 {
        use amdgpu::f1::AttrId;
        match self {
            GpuMetrics::F1C0(metrics) => average_without_sentinel(&[
                metrics.average_vclk0_frequency,
                metrics.average_vclk1_frequency,
            ]) as u32,
            GpuMetrics::F1C1(metrics) => average_without_sentinel(&[
                metrics.average_vclk0_frequency,
                metrics.average_vclk1_frequency,
            ]) as u32,
            GpuMetrics::F1C2(metrics) => average_without_sentinel(&[
                metrics.average_vclk0_frequency,
                metrics.average_vclk1_frequency,
            ]) as u32,
            GpuMetrics::F1C3(metrics) => average_without_sentinel(&[
                metrics.average_vclk0_frequency,
                metrics.average_vclk1_frequency,
            ]) as u32,
            GpuMetrics::F1C4(metrics) => average_without_sentinel(&metrics.current_vclk0) as u32,
            GpuMetrics::F1C5(metrics) => average_without_sentinel(&metrics.current_vclk0) as u32,
            GpuMetrics::F1C6(metrics) => average_without_sentinel(&metrics.current_vclk0) as u32,
            GpuMetrics::F1C7(metrics) => average_without_sentinel(&metrics.current_vclk0) as u32,
            GpuMetrics::F1C8(metrics) => average_without_sentinel(&metrics.current_vclk0) as u32,
            GpuMetrics::F1C9(metrics) => average_without_sentinel(
                &metrics
                    .attributes
                    .iter()
                    .find(|attr| attr.id == AttrId::CurrentVclk0)
                    .map(|val| val.get_as::<u16>())
                    .unwrap_or_default(),
            ) as u32,
            GpuMetrics::F2C0(metrics) => metrics.average_uclk_frequency as u32,
            GpuMetrics::F2C1(metrics) => metrics.average_uclk_frequency as u32,
            GpuMetrics::F2C2(metrics) => metrics.average_uclk_frequency as u32,
            GpuMetrics::F2C3(metrics) => metrics.average_uclk_frequency as u32,
            GpuMetrics::F2C4(metrics) => metrics.average_uclk_frequency as u32,
            GpuMetrics::F3C0(metrics) => metrics.average_uclk_frequency as u32,
        }
    }

    pub fn decoder_clock(&self) -> u32 {
        use amdgpu::f1::AttrId;
        match self {
            GpuMetrics::F1C0(metrics) => average_without_sentinel(&[
                metrics.average_dclk0_frequency,
                metrics.average_dclk1_frequency,
            ]) as u32,
            GpuMetrics::F1C1(metrics) => average_without_sentinel(&[
                metrics.average_dclk0_frequency,
                metrics.average_dclk1_frequency,
            ]) as u32,
            GpuMetrics::F1C2(metrics) => average_without_sentinel(&[
                metrics.average_dclk0_frequency,
                metrics.average_dclk1_frequency,
            ]) as u32,
            GpuMetrics::F1C3(metrics) => average_without_sentinel(&[
                metrics.average_dclk0_frequency,
                metrics.average_dclk1_frequency,
            ]) as u32,
            GpuMetrics::F1C4(metrics) => average_without_sentinel(&metrics.current_dclk0) as u32,
            GpuMetrics::F1C5(metrics) => average_without_sentinel(&metrics.current_dclk0) as u32,
            GpuMetrics::F1C6(metrics) => average_without_sentinel(&metrics.current_dclk0) as u32,
            GpuMetrics::F1C7(metrics) => average_without_sentinel(&metrics.current_dclk0) as u32,
            GpuMetrics::F1C8(metrics) => average_without_sentinel(&metrics.current_dclk0) as u32,
            GpuMetrics::F1C9(metrics) => average_without_sentinel(
                &metrics
                    .attributes
                    .iter()
                    .find(|attr| attr.id == AttrId::CurrentDclk0)
                    .map(|val| val.get_as::<u16>())
                    .unwrap_or_default(),
            ) as u32,
            GpuMetrics::F2C0(metrics) => metrics.average_uclk_frequency as u32,
            GpuMetrics::F2C1(metrics) => metrics.average_uclk_frequency as u32,
            GpuMetrics::F2C2(metrics) => metrics.average_uclk_frequency as u32,
            GpuMetrics::F2C3(metrics) => metrics.average_uclk_frequency as u32,
            GpuMetrics::F2C4(metrics) => metrics.average_uclk_frequency as u32,
            GpuMetrics::F3C0(metrics) => metrics.average_uclk_frequency as u32,
        }
    }

    pub fn power_milliwatt(&self) -> u32 {
        use amdgpu::f1::AttrId;
        match self {
            GpuMetrics::F1C0(metrics) => metrics.average_socket_power as u32 * 1000,
            GpuMetrics::F1C1(metrics) => metrics.average_socket_power as u32 * 1000,
            GpuMetrics::F1C2(metrics) => metrics.average_socket_power as u32 * 1000,
            GpuMetrics::F1C3(metrics) => metrics.average_socket_power as u32 * 1000,
            GpuMetrics::F1C4(metrics) => metrics.curr_socket_power as u32 * 1000,
            GpuMetrics::F1C5(metrics) => metrics.curr_socket_power as u32 * 1000,
            GpuMetrics::F1C6(metrics) => metrics.curr_socket_power as u32 * 1000,
            GpuMetrics::F1C7(metrics) => metrics.curr_socket_power as u32 * 1000,
            GpuMetrics::F1C8(metrics) => metrics.curr_socket_power as u32 * 1000,
            GpuMetrics::F1C9(metrics) => {
                average_without_sentinel(
                    &metrics
                        .attributes
                        .iter()
                        .find(|attr| attr.id == AttrId::CurrSocPower)
                        .map(|val| val.get_as::<u16>())
                        .unwrap_or_default(),
                ) as u32
                    * 1000
            }
            GpuMetrics::F2C0(metrics) => metrics.average_socket_power as u32 * 1000,
            GpuMetrics::F2C1(metrics) => metrics.average_socket_power as u32 * 1000,
            GpuMetrics::F2C2(metrics) => metrics.average_socket_power as u32 * 1000,
            GpuMetrics::F2C3(metrics) => metrics.average_socket_power as u32 * 1000,
            GpuMetrics::F2C4(metrics) => metrics.average_socket_power as u32 * 1000,
            GpuMetrics::F3C0(metrics) => metrics.average_socket_power as u32 * 1000,
        }
    }

    pub fn temperature(&self) -> i32 {
        use amdgpu::f1::AttrId;
        match self {
            GpuMetrics::F1C0(metrics) => metrics.temperature_hotspot as i32,
            GpuMetrics::F1C1(metrics) => metrics.temperature_hotspot as i32,
            GpuMetrics::F1C2(metrics) => metrics.temperature_hotspot as i32,
            GpuMetrics::F1C3(metrics) => metrics.temperature_hotspot as i32,
            GpuMetrics::F1C4(metrics) => metrics.temperature_hotspot as i32,
            GpuMetrics::F1C5(metrics) => metrics.temperature_hotspot as i32,
            GpuMetrics::F1C6(metrics) => metrics.temperature_hotspot as i32,
            GpuMetrics::F1C7(metrics) => metrics.temperature_hotspot as i32,
            GpuMetrics::F1C8(metrics) => metrics.temperature_hotspot as i32,
            GpuMetrics::F1C9(metrics) => average_without_sentinel(
                &metrics
                    .attributes
                    .iter()
                    .find(|attr| attr.id == AttrId::TemperatureHotspot)
                    .map(|val| val.get_as::<u16>())
                    .unwrap_or_default(),
            ) as i32,
            GpuMetrics::F2C0(metrics) => metrics.temperature_gfx as i32 / 100,
            GpuMetrics::F2C1(metrics) => metrics.temperature_gfx as i32 / 100,
            GpuMetrics::F2C2(metrics) => metrics.temperature_gfx as i32 / 100,
            GpuMetrics::F2C3(metrics) => metrics.temperature_gfx as i32 / 100,
            GpuMetrics::F2C4(metrics) => metrics.temperature_gfx as i32 / 100,
            GpuMetrics::F3C0(metrics) => metrics.temperature_gfx as i32 / 100,
        }
    }
}

fn average_without_sentinel(values: &[u16]) -> u16 {
    let mut accum = 0;
    let mut num = 0;
    for value in values.iter() {
        if *value != 0xFFFF {
            accum += value;
            num += 1;
        }
    }
    if num != 0 { accum / num } else { 0 }
}

fn from_bytes<T>(bytes: &[u8]) -> Option<T> {
    if bytes.len() != size_of::<T>() {
        return None;
    }

    unsafe { Some(std::ptr::read_unaligned(bytes.as_ptr() as *const T)) }
}

trait AmdgpuMetrics {
    fn from_bytes(bytes: &[u8]) -> anyhow::Result<GpuMetrics>;
}

/// Contains the struct definitions from the amdgpu driver.
mod amdgpu {
    use super::*;

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct Header {
        pub structure_size: u16,
        pub format_revision: u8,
        pub content_revision: u8,
    }

    pub mod f1 {
        use super::*;

        // === gpu_metrics_v1_0 ===

        #[repr(C)]
        #[derive(Debug, Clone)]
        pub struct C0 {
            common_header: Header,

            /* Driver attached timestamp (in ns) */
            pub system_clock_counter: u64,

            /* Temperature */
            pub temperature_edge: u16,
            pub temperature_hotspot: u16,
            pub temperature_mem: u16,
            pub temperature_vrgfx: u16,
            pub temperature_vrsoc: u16,
            pub temperature_vrmem: u16,

            /* Utilization */
            pub average_gfx_activity: u16,
            pub average_umc_activity: u16, // memory controller
            pub average_mm_activity: u16,  // UVD or VCN

            /* Power/Energy */
            pub average_socket_power: u16,
            pub energy_accumulator: u32,

            /* Average clocks */
            pub average_gfxclk_frequency: u16,
            pub average_socclk_frequency: u16,
            pub average_uclk_frequency: u16,
            pub average_vclk0_frequency: u16,
            pub average_dclk0_frequency: u16,
            pub average_vclk1_frequency: u16,
            pub average_dclk1_frequency: u16,

            /* Current clocks */
            pub current_gfxclk: u16,
            pub current_socclk: u16,
            pub current_uclk: u16,
            pub current_vclk0: u16,
            pub current_dclk0: u16,
            pub current_vclk1: u16,
            pub current_dclk1: u16,

            /* Throttle status */
            pub throttle_status: u32,

            /* Fans */
            pub current_fan_speed: u16,

            /* Link width/speed */
            pub pcie_link_width: u8,
            pub pcie_link_speed: u8, // in 0.1 GT/s
        }

        impl AmdgpuMetrics for C0 {
            fn from_bytes(bytes: &[u8]) -> anyhow::Result<GpuMetrics> {
                tracing::debug!("Reading format 1 content 0");
                let bytes = from_bytes::<Self>(bytes);
                Ok(GpuMetrics::F1C0(bytes.ok_or_else(|| {
                    anyhow::anyhow!("gpu_metrics file struct malformed")
                })?))
            }
        }

        // === gpu_metrics_v1_1 ===

        const NUM_HBM_INSTANCES: usize = 4;
        #[repr(C)]
        #[derive(Debug, Clone)]
        pub struct C1 {
            common_header: Header,

            /* Temperature */
            pub temperature_edge: u16,
            pub temperature_hotspot: u16,
            pub temperature_mem: u16,
            pub temperature_vrgfx: u16,
            pub temperature_vrsoc: u16,
            pub temperature_vrmem: u16,

            /* Utilization */
            pub average_gfx_activity: u16,
            pub average_umc_activity: u16, // memory controller
            pub average_mm_activity: u16,  // UVD or VCN

            /* Power/Energy */
            pub average_socket_power: u16,
            pub energy_accumulator: u32,

            /* Driver attached timestamp (in ns) */
            pub system_clock_counter: u64,

            /* Average clocks */
            pub average_gfxclk_frequency: u16,
            pub average_socclk_frequency: u16,
            pub average_uclk_frequency: u16,
            pub average_vclk0_frequency: u16,
            pub average_dclk0_frequency: u16,
            pub average_vclk1_frequency: u16,
            pub average_dclk1_frequency: u16,

            /* Current clocks */
            pub current_gfxclk: u16,
            pub current_socclk: u16,
            pub current_uclk: u16,
            pub current_vclk0: u16,
            pub current_dclk0: u16,
            pub current_vclk1: u16,
            pub current_dclk1: u16,

            /* Throttle status */
            pub throttle_status: u32,

            /* Fans */
            pub current_fan_speed: u16,

            /* Link width/speed */
            pub pcie_link_width: u8,
            pub pcie_link_speed: u8, // in 0.1 GT/s

            padding: u16,

            pub gfx_activity_acc: u32,
            pub mem_activity_acc: u32,

            pub temperature_hbm: [u16; NUM_HBM_INSTANCES],
        }

        impl AmdgpuMetrics for C1 {
            fn from_bytes(bytes: &[u8]) -> anyhow::Result<GpuMetrics> {
                tracing::debug!("Reading format 1 content 1");
                let bytes = from_bytes::<Self>(bytes);
                Ok(GpuMetrics::F1C1(bytes.ok_or_else(|| {
                    anyhow::anyhow!("gpu_metrics file struct malformed")
                })?))
            }
        }

        // === gpu_metrics_v1_2 ===

        #[repr(C)]
        #[derive(Debug, Clone)]
        pub struct C2 {
            common_header: Header,

            /* Temperature */
            pub temperature_edge: u16,
            pub temperature_hotspot: u16,
            pub temperature_mem: u16,
            pub temperature_vrgfx: u16,
            pub temperature_vrsoc: u16,
            pub temperature_vrmem: u16,

            /* Utilization */
            pub average_gfx_activity: u16,
            pub average_umc_activity: u16, // memory controller
            pub average_mm_activity: u16,  // UVD or VCN

            /* Power/Energy */
            pub average_socket_power: u16,
            pub energy_accumulator: u64,

            /* Driver attached timestamp (in ns) */
            pub system_clock_counter: u64,

            /* Average clocks */
            pub average_gfxclk_frequency: u16,
            pub average_socclk_frequency: u16,
            pub average_uclk_frequency: u16,
            pub average_vclk0_frequency: u16,
            pub average_dclk0_frequency: u16,
            pub average_vclk1_frequency: u16,
            pub average_dclk1_frequency: u16,

            /* Current clocks */
            pub current_gfxclk: u16,
            pub current_socclk: u16,
            pub current_uclk: u16,
            pub current_vclk0: u16,
            pub current_dclk0: u16,
            pub current_vclk1: u16,
            pub current_dclk1: u16,

            /* Throttle status (ASIC dependent) */
            pub throttle_status: u32,

            /* Fans */
            pub current_fan_speed: u16,

            /* Link width/speed */
            pub pcie_link_width: u16,
            pub pcie_link_speed: u16, // in 0.1 GT/s

            padding: u16,

            pub gfx_activity_acc: u32,
            pub mem_activity_acc: u32,

            pub temperature_hbm: [u16; NUM_HBM_INSTANCES],

            /* PMFW attached timestamp (10ns resolution) */
            pub firmware_timestamp: u64,
        }

        impl AmdgpuMetrics for C2 {
            fn from_bytes(bytes: &[u8]) -> anyhow::Result<GpuMetrics> {
                tracing::debug!("Reading format 1 content 2");
                let bytes = from_bytes::<Self>(bytes);
                Ok(GpuMetrics::F1C2(bytes.ok_or_else(|| {
                    anyhow::anyhow!("gpu_metrics file struct malformed")
                })?))
            }
        }

        // === gpu_metrics_v1_3 ===

        #[repr(C)]
        #[derive(Debug, Clone)]
        pub struct C3 {
            common_header: Header,

            /* Temperature */
            pub temperature_edge: u16,
            pub temperature_hotspot: u16,
            pub temperature_mem: u16,
            pub temperature_vrgfx: u16,
            pub temperature_vrsoc: u16,
            pub temperature_vrmem: u16,

            /* Utilization */
            pub average_gfx_activity: u16,
            pub average_umc_activity: u16, // memory controller
            pub average_mm_activity: u16,  // UVD or VCN

            /* Power/Energy */
            pub average_socket_power: u16,
            pub energy_accumulator: u64,

            /* Driver attached timestamp (in ns) */
            pub system_clock_counter: u64,

            /* Average clocks */
            pub average_gfxclk_frequency: u16,
            pub average_socclk_frequency: u16,
            pub average_uclk_frequency: u16,
            pub average_vclk0_frequency: u16,
            pub average_dclk0_frequency: u16,
            pub average_vclk1_frequency: u16,
            pub average_dclk1_frequency: u16,

            /* Current clocks */
            pub current_gfxclk: u16,
            pub current_socclk: u16,
            pub current_uclk: u16,
            pub current_vclk0: u16,
            pub current_dclk0: u16,
            pub current_vclk1: u16,
            pub current_dclk1: u16,

            /* Throttle status */
            pub throttle_status: u32,

            /* Fans */
            pub current_fan_speed: u16,

            /* Link width/speed */
            pub pcie_link_width: u16,
            pub pcie_link_speed: u16, // in 0.1 GT/s

            padding: u16,

            pub gfx_activity_acc: u32,
            pub mem_activity_acc: u32,

            pub temperature_hbm: [u16; NUM_HBM_INSTANCES],

            /* PMFW attached timestamp (10ns resolution) */
            pub firmware_timestamp: u64,

            /* Voltage (mV) */
            pub voltage_soc: u16,
            pub voltage_gfx: u16,
            pub voltage_mem: u16,

            padding1: u16,

            /* Throttle status (ASIC independent) */
            pub indep_throttle_status: u64,
        }

        impl AmdgpuMetrics for C3 {
            fn from_bytes(bytes: &[u8]) -> anyhow::Result<GpuMetrics> {
                tracing::debug!("Reading format 1 content 3");
                let bytes = from_bytes::<Self>(bytes);
                Ok(GpuMetrics::F1C3(bytes.ok_or_else(|| {
                    anyhow::anyhow!("gpu_metrics file struct malformed")
                })?))
            }
        }

        // === gpu_metrics_v1_4 ===

        const NUM_XGMI_LINKS: usize = 8;
        const MAX_GFX_CLKS: usize = 8;
        const MAX_CLKS: usize = 4;
        const NUM_VCN: usize = 4;

        #[repr(C)]
        #[derive(Debug, Clone)]
        pub struct C4 {
            common_header: Header,

            /* Temperature (Celsius) */
            pub temperature_hotspot: u16,
            pub temperature_mem: u16,
            pub temperature_vrsoc: u16,

            /* Power (Watts) */
            pub curr_socket_power: u16,

            /* Utilization (%) */
            pub average_gfx_activity: u16,
            pub average_umc_activity: u16, // memory controller
            pub vcn_activity: [u16; NUM_VCN],

            /* Energy (15.259uJ (2^-16) units) */
            pub energy_accumulator: u64,

            /* Driver attached timestamp (in ns) */
            pub system_clock_counter: u64,

            /* Throttle status */
            pub throttle_status: u32,

            /* Clock Lock Status. Each bit corresponds to clock instance */
            pub gfxclk_lock_status: u32,

            /* Link width (number of lanes) and speed (in 0.1 GT/s) */
            pub pcie_link_width: u16,
            pub pcie_link_speed: u16,

            /* XGMI bus width and bitrate (in Gbps) */
            pub xgmi_link_width: u16,
            pub xgmi_link_speed: u16,

            /* Utilization Accumulated (%) */
            pub gfx_activity_acc: u32,
            pub mem_activity_acc: u32,

            /*PCIE accumulated bandwidth (GB/sec) */
            pub pcie_bandwidth_acc: u64,

            /*PCIE instantaneous bandwidth (GB/sec) */
            pub pcie_bandwidth_inst: u64,

            /* PCIE L0 to recovery state transition accumulated count */
            pub pcie_l0_to_recov_count_acc: u64,

            /* PCIE replay accumulated count */
            pub pcie_replay_count_acc: u64,

            /* PCIE replay rollover accumulated count */
            pub pcie_replay_rover_count_acc: u64,

            /* XGMI accumulated data transfer size(KiloBytes) */
            pub xgmi_read_data_acc: [u64; NUM_XGMI_LINKS],
            pub xgmi_write_data_acc: [u64; NUM_XGMI_LINKS],

            /* PMFW attached timestamp (10ns resolution) */
            pub firmware_timestamp: u64,

            /* Current clocks (Mhz) */
            pub current_gfxclk: [u16; MAX_GFX_CLKS],
            pub current_socclk: [u16; MAX_CLKS],
            pub current_vclk0: [u16; MAX_CLKS],
            pub current_dclk0: [u16; MAX_CLKS],
            pub current_uclk: u16,

            padding: u16,
        }

        impl AmdgpuMetrics for C4 {
            fn from_bytes(bytes: &[u8]) -> anyhow::Result<GpuMetrics> {
                tracing::debug!("Reading format 1 content 4");
                let bytes = from_bytes::<Self>(bytes);
                Ok(GpuMetrics::F1C4(bytes.ok_or_else(|| {
                    anyhow::anyhow!("gpu_metrics file struct malformed")
                })?))
            }
        }

        // === gpu_metrics_v1_5 ===

        const NUM_JPEG_ENG: usize = 32;

        #[repr(C)]
        #[derive(Debug, Clone)]
        pub struct C5 {
            common_header: Header,

            /* Temperature (Celsius) */
            pub temperature_hotspot: u16,
            pub temperature_mem: u16,
            pub temperature_vrsoc: u16,

            /* Power (Watts) */
            pub curr_socket_power: u16,

            /* Utilization (%) */
            pub average_gfx_activity: u16,
            pub average_umc_activity: u16, // memory controller
            pub vcn_activity: [u16; NUM_VCN],
            pub jpeg_activity: [u16; NUM_JPEG_ENG],

            /* Energy (15.259uJ (2^-16) units) */
            pub energy_accumulator: u64,

            /* Driver attached timestamp (in ns) */
            pub system_clock_counter: u64,

            /* Throttle status */
            pub throttle_status: u32,

            /* Clock Lock Status. Each bit corresponds to clock instance */
            pub gfxclk_lock_status: u32,

            /* Link width (number of lanes) and speed (in 0.1 GT/s) */
            pub pcie_link_width: u16,
            pub pcie_link_speed: u16,

            /* XGMI bus width and bitrate (in Gbps) */
            pub xgmi_link_width: u16,
            pub xgmi_link_speed: u16,

            /* Utilization Accumulated (%) */
            pub gfx_activity_acc: u32,
            pub mem_activity_acc: u32,

            /*PCIE accumulated bandwidth (GB/sec) */
            pub pcie_bandwidth_acc: u64,

            /*PCIE instantaneous bandwidth (GB/sec) */
            pub pcie_bandwidth_inst: u64,

            /* PCIE L0 to recovery state transition accumulated count */
            pub pcie_l0_to_recov_count_acc: u64,

            /* PCIE replay accumulated count */
            pub pcie_replay_count_acc: u64,

            /* PCIE replay rollover accumulated count */
            pub pcie_replay_rover_count_acc: u64,

            /* PCIE NAK sent  accumulated count */
            pub pcie_nak_sent_count_acc: u32,

            /* PCIE NAK received accumulated count */
            pub pcie_nak_rcvd_count_acc: u32,

            /* XGMI accumulated data transfer size(KiloBytes) */
            pub xgmi_read_data_acc: [u64; NUM_XGMI_LINKS],
            pub xgmi_write_data_acc: [u64; NUM_XGMI_LINKS],

            /* PMFW attached timestamp (10ns resolution) */
            pub firmware_timestamp: u64,

            /* Current clocks (Mhz) */
            pub current_gfxclk: [u16; MAX_GFX_CLKS],
            pub current_socclk: [u16; MAX_CLKS],
            pub current_vclk0: [u16; MAX_CLKS],
            pub current_dclk0: [u16; MAX_CLKS],
            pub current_uclk: u16,

            padding: u16,
        }

        impl AmdgpuMetrics for C5 {
            fn from_bytes(bytes: &[u8]) -> anyhow::Result<GpuMetrics> {
                tracing::debug!("Reading format 1 content 5");
                let bytes = from_bytes::<Self>(bytes);
                Ok(GpuMetrics::F1C5(bytes.ok_or_else(|| {
                    anyhow::anyhow!("gpu_metrics file struct malformed")
                })?))
            }
        }

        // === gpu_metrics_v1_6 ===

        const MAX_XCC: usize = 8;
        const NUM_XCP: usize = 8;
        #[repr(C)]
        #[derive(Debug, Clone)]
        pub struct XcpMetricsV1 {
            /* Utilization Instantaneous (%) */
            pub gfx_busy_inst: [u32; MAX_XCC],
            pub jpeg_busy: [u16; NUM_JPEG_ENG],
            pub vcn_busy: [u16; NUM_VCN],
            /* Utilization Accumulated (%) */
            pub gfx_busy_acc: [u64; MAX_XCC],
        }

        #[repr(C)]
        #[derive(Debug, Clone)]
        pub struct C6 {
            common_header: Header,

            /* Temperature (Celsius) */
            pub temperature_hotspot: u16,
            pub temperature_mem: u16,
            pub temperature_vrsoc: u16,

            /* Power (Watts) */
            pub curr_socket_power: u16,

            /* Utilization (%) */
            pub average_gfx_activity: u16,
            pub average_umc_activity: u16, // memory controller

            /* Energy (15.259uJ (2^-16) units) */
            pub energy_accumulator: u64,

            /* Driver attached timestamp (in ns) */
            pub system_clock_counter: u64,

            /* Accumulation cycle counter */
            pub accumulation_counter: u32,

            /* Accumulated throttler residencies */
            pub prochot_residency_acc: u32,
            pub ppt_residency_acc: u32,
            pub socket_thm_residency_acc: u32,
            pub vr_thm_residency_acc: u32,
            pub hbm_thm_residency_acc: u32,

            /* Clock Lock Status. Each bit corresponds to clock instance */
            pub gfxclk_lock_status: u32,

            /* Link width (number of lanes) and speed (in 0.1 GT/s) */
            pub pcie_link_width: u16,
            pub pcie_link_speed: u16,

            /* XGMI bus width and bitrate (in Gbps) */
            pub xgmi_link_width: u16,
            pub xgmi_link_speed: u16,

            /* Utilization Accumulated (%) */
            pub gfx_activity_acc: u32,
            pub mem_activity_acc: u32,

            /*PCIE accumulated bandwidth (GB/sec) */
            pub pcie_bandwidth_acc: u64,

            /*PCIE instantaneous bandwidth (GB/sec) */
            pub pcie_bandwidth_inst: u64,

            /* PCIE L0 to recovery state transition accumulated count */
            pub pcie_l0_to_recov_count_acc: u64,

            /* PCIE replay accumulated count */
            pub pcie_replay_count_acc: u64,

            /* PCIE replay rollover accumulated count */
            pub pcie_replay_rover_count_acc: u64,

            /* PCIE NAK sent  accumulated count */
            pub pcie_nak_sent_count_acc: u32,

            /* PCIE NAK received accumulated count */
            pub pcie_nak_rcvd_count_acc: u32,

            /* XGMI accumulated data transfer size(KiloBytes) */
            pub xgmi_read_data_acc: [u64; NUM_XGMI_LINKS],
            pub xgmi_write_data_acc: [u64; NUM_XGMI_LINKS],

            /* PMFW attached timestamp (10ns resolution) */
            pub firmware_timestamp: u64,

            /* Current clocks (Mhz) */
            pub current_gfxclk: [u16; MAX_GFX_CLKS],
            pub current_socclk: [u16; MAX_CLKS],
            pub current_vclk0: [u16; MAX_CLKS],
            pub current_dclk0: [u16; MAX_CLKS],
            pub current_uclk: u16,

            /* Number of current partition */
            pub num_partition: u16,

            /* XCP metrics stats */
            pub xcp_stats: [XcpMetricsV1; NUM_XCP],

            /* PCIE other end recovery counter */
            pub pcie_lc_perf_other_end_recovery: u32,
        }

        impl AmdgpuMetrics for C6 {
            fn from_bytes(bytes: &[u8]) -> anyhow::Result<GpuMetrics> {
                tracing::debug!("Reading format 1 content 6");
                let bytes = from_bytes::<Self>(bytes);
                Ok(GpuMetrics::F1C6(bytes.ok_or_else(|| {
                    anyhow::anyhow!("gpu_metrics file struct malformed")
                })?))
            }
        }

        // === gpu_metrics_v1_7 ===

        #[repr(C)]
        #[derive(Debug, Clone)]
        pub struct XcpMetricsV1_1 {
            /* Utilization Instantaneous (%) */
            pub gfx_busy_inst: [u32; MAX_XCC],
            pub jpeg_busy: [u16; NUM_JPEG_ENG],
            pub vcn_busy: [u16; NUM_VCN],

            /* Utilization Accumulated (%) */
            pub gfx_busy_acc: [u64; MAX_XCC],

            /* Total App Clock Counter Accumulated */
            pub gfx_below_host_limit_acc: [u64; MAX_XCC],
        }

        #[repr(C)]
        #[derive(Debug, Clone)]
        pub struct C7 {
            common_header: Header,

            /* Temperature (Celsius) */
            pub temperature_hotspot: u16,
            pub temperature_mem: u16,
            pub temperature_vrsoc: u16,

            /* Power (Watts) */
            pub curr_socket_power: u16,

            /* Utilization (%) */
            pub average_gfx_activity: u16,
            pub average_umc_activity: u16, // memory controller

            /* VRAM max bandwidthi (in GB/sec) at max memory clock */
            pub mem_max_bandwidth: u64,

            /* Energy (15.259uJ (2^-16) units) */
            pub energy_accumulator: u64,

            /* Driver attached timestamp (in ns) */
            pub system_clock_counter: u64,

            /* Accumulation cycle counter */
            pub accumulation_counter: u32,

            /* Accumulated throttler residencies */
            pub prochot_residency_acc: u32,
            pub ppt_residency_acc: u32,
            pub socket_thm_residency_acc: u32,
            pub vr_thm_residency_acc: u32,
            pub hbm_thm_residency_acc: u32,

            /* Clock Lock Status. Each bit corresponds to clock instance */
            pub gfxclk_lock_status: u32,

            /* Link width (number of lanes) and speed (in 0.1 GT/s) */
            pub pcie_link_width: u16,
            pub pcie_link_speed: u16,

            /* XGMI bus width and bitrate (in Gbps) */
            pub xgmi_link_width: u16,
            pub xgmi_link_speed: u16,

            /* Utilization Accumulated (%) */
            pub gfx_activity_acc: u32,
            pub mem_activity_acc: u32,

            /*PCIE accumulated bandwidth (GB/sec) */
            pub pcie_bandwidth_acc: u64,

            /*PCIE instantaneous bandwidth (GB/sec) */
            pub pcie_bandwidth_inst: u64,

            /* PCIE L0 to recovery state transition accumulated count */
            pub pcie_l0_to_recov_count_acc: u64,

            /* PCIE replay accumulated count */
            pub pcie_replay_count_acc: u64,

            /* PCIE replay rollover accumulated count */
            pub pcie_replay_rover_count_acc: u64,

            /* PCIE NAK sent  accumulated count */
            pub pcie_nak_sent_count_acc: u32,

            /* PCIE NAK received accumulated count */
            pub pcie_nak_rcvd_count_acc: u32,

            /* XGMI accumulated data transfer size(KiloBytes) */
            pub xgmi_read_data_acc: [u64; NUM_XGMI_LINKS],
            pub xgmi_write_data_acc: [u64; NUM_XGMI_LINKS],

            /* XGMI link status(active/inactive) */
            pub xgmi_link_status: [u16; NUM_XGMI_LINKS],

            padding: u16,

            /* PMFW attached timestamp (10ns resolution) */
            pub firmware_timestamp: u64,

            /* Current clocks (Mhz) */
            pub current_gfxclk: [u16; MAX_GFX_CLKS],
            pub current_socclk: [u16; MAX_CLKS],
            pub current_vclk0: [u16; MAX_CLKS],
            pub current_dclk0: [u16; MAX_CLKS],
            pub current_uclk: u16,

            /* Number of current partition */
            pub num_partition: u16,

            /* XCP metrics stats */
            pub xcp_stats: [XcpMetricsV1_1; NUM_XCP],

            /* PCIE other end recovery counter */
            pub pcie_lc_perf_other_end_recovery: u32,
        }

        impl AmdgpuMetrics for C7 {
            fn from_bytes(bytes: &[u8]) -> anyhow::Result<GpuMetrics> {
                tracing::debug!("Reading format 1 content 7");
                let bytes = from_bytes::<Self>(bytes);
                Ok(GpuMetrics::F1C7(bytes.ok_or_else(|| {
                    anyhow::anyhow!("gpu_metrics file struct malformed")
                })?))
            }
        }

        // === gpu_metrics_v1_8 ===

        const NUM_JPEG_ENG_V1: usize = 40;
        #[repr(C)]
        #[derive(Debug, Clone)]
        pub struct XcpMetricsV1_2 {
            /* Utilization Instantaneous (%) */
            gfx_busy_inst: [u32; MAX_XCC],
            pub jpeg_busy: [u16; NUM_JPEG_ENG_V1],
            pub vcn_busy: [u16; NUM_VCN],

            /* Utilization Accumulated (%) */
            pub gfx_busy_acc: [u64; MAX_XCC],

            /* Total App Clock Counter Accumulated */
            pub gfx_below_host_limit_ppt_acc: [u64; MAX_XCC],
            pub gfx_below_host_limit_thm_acc: [u64; MAX_XCC],
            pub gfx_low_utilization_acc: [u64; MAX_XCC],
            pub gfx_below_host_limit_total_acc: [u64; MAX_XCC],
        }

        #[repr(C)]
        #[derive(Debug, Clone)]
        pub struct C8 {
            common_header: Header,

            /* Temperature (Celsius) */
            pub temperature_hotspot: u16,
            pub temperature_mem: u16,
            pub temperature_vrsoc: u16,

            /* Power (Watts) */
            pub curr_socket_power: u16,

            /* Utilization (%) */
            pub average_gfx_activity: u16,
            pub average_umc_activity: u16, // memory controller

            /* VRAM max bandwidthi (in GB/sec) at max memory clock */
            pub mem_max_bandwidth: u64,

            /* Energy (15.259uJ (2^-16) units) */
            pub energy_accumulator: u64,

            /* Driver attached timestamp (in ns) */
            pub system_clock_counter: u64,

            /* Accumulation cycle counter */
            pub accumulation_counter: u32,

            /* Accumulated throttler residencies */
            pub prochot_residency_acc: u32,
            pub ppt_residency_acc: u32,
            pub socket_thm_residency_acc: u32,
            pub vr_thm_residency_acc: u32,
            pub hbm_thm_residency_acc: u32,

            /* Clock Lock Status. Each bit corresponds to clock instance */
            pub gfxclk_lock_status: u32,

            /* Link width (number of lanes) and speed (in 0.1 GT/s) */
            pub pcie_link_width: u16,
            pub pcie_link_speed: u16,

            /* XGMI bus width and bitrate (in Gbps) */
            pub xgmi_link_width: u16,
            pub xgmi_link_speed: u16,

            /* Utilization Accumulated (%) */
            pub gfx_activity_acc: u32,
            pub mem_activity_acc: u32,

            /*PCIE accumulated bandwidth (GB/sec) */
            pub pcie_bandwidth_acc: u64,

            /*PCIE instantaneous bandwidth (GB/sec) */
            pub pcie_bandwidth_inst: u64,

            /* PCIE L0 to recovery state transition accumulated count */
            pub pcie_l0_to_recov_count_acc: u64,

            /* PCIE replay accumulated count */
            pub pcie_replay_count_acc: u64,

            /* PCIE replay rollover accumulated count */
            pub pcie_replay_rover_count_acc: u64,

            /* PCIE NAK sent  accumulated count */
            pub pcie_nak_sent_count_acc: u32,

            /* PCIE NAK received accumulated count */
            pub pcie_nak_rcvd_count_acc: u32,

            /* XGMI accumulated data transfer size(KiloBytes) */
            pub xgmi_read_data_acc: [u64; NUM_XGMI_LINKS],
            pub xgmi_write_data_acc: [u64; NUM_XGMI_LINKS],

            /* XGMI link status(active/inactive) */
            pub xgmi_link_status: [u16; NUM_XGMI_LINKS],

            padding: u16,

            /* PMFW attached timestamp (10ns resolution) */
            pub firmware_timestamp: u64,

            /* Current clocks (Mhz) */
            pub current_gfxclk: [u16; MAX_GFX_CLKS],
            pub current_socclk: [u16; MAX_CLKS],
            pub current_vclk0: [u16; MAX_CLKS],
            pub current_dclk0: [u16; MAX_CLKS],
            pub current_uclk: u16,

            /* Number of current partition */
            pub num_partition: u16,

            /* XCP metrics stats */
            pub xcp_stats: [XcpMetricsV1_2; NUM_XCP],

            /* PCIE other end recovery counter */
            pub pcie_lc_perf_other_end_recovery: u32,
        }

        impl AmdgpuMetrics for C8 {
            fn from_bytes(bytes: &[u8]) -> anyhow::Result<GpuMetrics> {
                tracing::debug!("Reading format 1 content 8");
                let bytes = from_bytes::<Self>(bytes);
                Ok(GpuMetrics::F1C8(bytes.ok_or_else(|| {
                    anyhow::anyhow!("gpu_metrics file struct malformed")
                })?))
            }
        }

        // === gpu_metrics_v1_9 ===
        // Version 9 is special because it switches to a FAM pattern
        // I'm also gonna be deadass, I am not 100% sure if this is correct because I don't have an MI300 AI accelerator to test it.
        // So I'm just winging it based on some very surface level reading of the AMD SMU source code, I'm not 100% sure if it's packed or not.

        const ATTR_TYPE_MASK: u64 = 0x00F00000;
        const ATTR_TYPE_SHIFT: u64 = 20;
        const ATTR_ID_MASK: u64 = 0x000FFC00;
        const ATTR_ID_SHIFT: u64 = 10;
        const ATTR_INST_MASK: u64 = 0x000003FF;

        #[derive(Debug, Clone)]
        pub enum AttrValue {
            U8 { values: Vec<u8> },
            S8 { values: Vec<i8> },
            U16 { values: Vec<u16> },
            S16 { values: Vec<i16> },
            U32 { values: Vec<u32> },
            S32 { values: Vec<i32> },
            U64 { values: Vec<u64> },
            S64 { values: Vec<i64> },
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum AttrId {
            TemperatureHotspot,
            TemperatureMem,
            TemperatureVrsoc,
            CurrSocPower,
            AverageGfxActivity,
            AverageUmcActivity,
            MemMaxBandwidth,
            EnergyAccumulator,
            SystemClockCounter,
            AccumulationCounter,
            ProchotResidencyAcc,
            PptResidencyAcc,
            SocketThmResidencyAcc,
            VrThmResidencyAcc,
            HbmThmResidencyAcc,
            GfxclkLockStatus,
            PcieLinkWidth,
            PcieLinkSpeed,
            XgmiLinkWidth,
            XgmiLinkSpeed,
            GfxActivityAcc,
            MemActivityAcc,
            PcieBandwidthAcc,
            PcieBandwidthInst,
            PcieL0ToRecovCountAcc,
            PcieReplayCountAcc,
            PcieReplayRoverCountAcc,
            PcieNakSentCountAcc,
            PcieNakRcvdCountAcc,
            XgmiReadDataAcc,
            XgmiWriteDataAcc,
            XgmiLinkStatus,
            FirmwareTimestamp,
            CurrentGfxclk,
            CurrentSocclk,
            CurrentVclk0,
            CurrentDclk0,
            CurrentUclk,
            NumPartition,
            PcieLcPerfOtherEndRecovery,
            GfxBusyInst,
            JpegBusy,
            VcnBusy,
            GfxBusyAcc,
            GfxBelowHostLimitPptAcc,
            GfxBelowHostLimitThmAcc,
            GfxLowUtilizationAcc,
            GfxBelowHostLimitTotalAcc,
        }

        impl TryFrom<u64> for AttrId {
            type Error = anyhow::Error;

            fn try_from(value: u64) -> anyhow::Result<Self> {
                match value {
                    00 => Ok(Self::TemperatureHotspot),
                    01 => Ok(Self::TemperatureMem),
                    02 => Ok(Self::TemperatureVrsoc),
                    03 => Ok(Self::CurrSocPower),
                    04 => Ok(Self::AverageGfxActivity),
                    05 => Ok(Self::AverageUmcActivity),
                    06 => Ok(Self::MemMaxBandwidth),
                    07 => Ok(Self::EnergyAccumulator),
                    08 => Ok(Self::SystemClockCounter),
                    09 => Ok(Self::AccumulationCounter),
                    10 => Ok(Self::ProchotResidencyAcc),
                    11 => Ok(Self::PptResidencyAcc),
                    12 => Ok(Self::SocketThmResidencyAcc),
                    13 => Ok(Self::VrThmResidencyAcc),
                    14 => Ok(Self::HbmThmResidencyAcc),
                    15 => Ok(Self::GfxclkLockStatus),
                    16 => Ok(Self::PcieLinkWidth),
                    17 => Ok(Self::PcieLinkSpeed),
                    18 => Ok(Self::XgmiLinkWidth),
                    19 => Ok(Self::XgmiLinkSpeed),
                    20 => Ok(Self::GfxActivityAcc),
                    21 => Ok(Self::MemActivityAcc),
                    22 => Ok(Self::PcieBandwidthAcc),
                    23 => Ok(Self::PcieBandwidthInst),
                    24 => Ok(Self::PcieL0ToRecovCountAcc),
                    25 => Ok(Self::PcieReplayCountAcc),
                    26 => Ok(Self::PcieReplayRoverCountAcc),
                    27 => Ok(Self::PcieNakSentCountAcc),
                    28 => Ok(Self::PcieNakRcvdCountAcc),
                    29 => Ok(Self::XgmiReadDataAcc),
                    30 => Ok(Self::XgmiWriteDataAcc),
                    31 => Ok(Self::XgmiLinkStatus),
                    32 => Ok(Self::FirmwareTimestamp),
                    33 => Ok(Self::CurrentGfxclk),
                    34 => Ok(Self::CurrentSocclk),
                    35 => Ok(Self::CurrentVclk0),
                    36 => Ok(Self::CurrentDclk0),
                    37 => Ok(Self::CurrentUclk),
                    38 => Ok(Self::NumPartition),
                    39 => Ok(Self::PcieLcPerfOtherEndRecovery),
                    40 => Ok(Self::GfxBusyInst),
                    41 => Ok(Self::JpegBusy),
                    42 => Ok(Self::VcnBusy),
                    43 => Ok(Self::GfxBusyAcc),
                    44 => Ok(Self::GfxBelowHostLimitPptAcc),
                    45 => Ok(Self::GfxBelowHostLimitThmAcc),
                    46 => Ok(Self::GfxLowUtilizationAcc),
                    47 => Ok(Self::GfxBelowHostLimitTotalAcc),
                    _ => Err(anyhow::anyhow!("invalid attr_id value: {value}")),
                }
            }
        }

        #[derive(Debug, Clone)]
        pub struct Attribute {
            pub id: AttrId,
            pub val: AttrValue,
        }

        impl Attribute {
            pub fn get_as<T>(&self) -> Vec<T>
            where
                T: Copy + 'static + Default,
                u8: AsPrimitive<T>,
                i8: AsPrimitive<T>,
                u16: AsPrimitive<T>,
                i16: AsPrimitive<T>,
                u32: AsPrimitive<T>,
                i32: AsPrimitive<T>,
                u64: AsPrimitive<T>,
                i64: AsPrimitive<T>,
            {
                match &self.val {
                    AttrValue::U8 { values } => values
                        .iter()
                        .cloned()
                        .map(|val| val.as_())
                        .collect::<Vec<T>>(),
                    AttrValue::S8 { values } => values
                        .iter()
                        .cloned()
                        .map(|val| val.as_())
                        .collect::<Vec<T>>(),
                    AttrValue::U16 { values } => values
                        .iter()
                        .cloned()
                        .map(|val| val.as_())
                        .collect::<Vec<T>>(),
                    AttrValue::S16 { values } => values
                        .iter()
                        .cloned()
                        .map(|val| val.as_())
                        .collect::<Vec<T>>(),
                    AttrValue::U32 { values } => values
                        .iter()
                        .cloned()
                        .map(|val| val.as_())
                        .collect::<Vec<T>>(),
                    AttrValue::S32 { values } => values
                        .iter()
                        .cloned()
                        .map(|val| val.as_())
                        .collect::<Vec<T>>(),
                    AttrValue::U64 { values } => values
                        .iter()
                        .cloned()
                        .map(|val| val.as_())
                        .collect::<Vec<T>>(),
                    AttrValue::S64 { values } => values
                        .iter()
                        .cloned()
                        .map(|val| val.as_())
                        .collect::<Vec<T>>(),
                }
            }
        }

        #[derive(Debug, Clone)]
        pub struct C9 {
            pub attributes: Vec<Attribute>,
        }

        impl AmdgpuMetrics for C9 {
            fn from_bytes(bytes: &[u8]) -> anyhow::Result<GpuMetrics> {
                tracing::debug!("Reading format 1 content 9");
                use std::io::Read;
                let mut cursor = std::io::Cursor::new(bytes);
                let mut header_buf = [0u8; size_of::<Header>()];
                cursor.read_exact(&mut header_buf)?;
                let attr_count_buf = [0u8; size_of::<i32>()];
                cursor.read_exact(&mut header_buf)?;
                let attr_count = i32::from_ne_bytes(attr_count_buf);
                let mut attributes = Vec::new();
                for _ in 0..attr_count {
                    // Read the attribute encoding description (packed in 8 bytes)
                    let mut encoding_buf = [0u8; size_of::<u64>()];
                    cursor.read_exact(&mut encoding_buf)?;
                    let encoding = u64::from_ne_bytes(encoding_buf);
                    let ty = (encoding & ATTR_TYPE_MASK) >> ATTR_TYPE_SHIFT;
                    let id = AttrId::try_from((encoding & ATTR_ID_MASK) >> ATTR_ID_SHIFT)?;
                    let instance_count = encoding & ATTR_INST_MASK;

                    // Read the attribute values
                    let val = match ty {
                        0 => AttrValue::U8 {
                            values: {
                                let mut values = Vec::new();
                                for _ in 0..instance_count {
                                    let mut val_buf = [0u8; size_of::<u8>()];
                                    cursor.read_exact(&mut val_buf)?;
                                    let val = u8::from_be_bytes(val_buf);
                                    values.push(val);
                                }
                                values
                            },
                        },
                        1 => AttrValue::S8 {
                            values: {
                                let mut values = Vec::new();
                                for _ in 0..instance_count {
                                    let mut val_buf = [0u8; size_of::<i8>()];
                                    cursor.read_exact(&mut val_buf)?;
                                    let val = i8::from_be_bytes(val_buf);
                                    values.push(val);
                                }
                                values
                            },
                        },
                        2 => AttrValue::U16 {
                            values: {
                                let mut values = Vec::new();
                                for _ in 0..instance_count {
                                    let mut val_buf = [0u8; size_of::<u16>()];
                                    cursor.read_exact(&mut val_buf)?;
                                    let val = u16::from_be_bytes(val_buf);
                                    values.push(val);
                                }
                                values
                            },
                        },
                        3 => AttrValue::S16 {
                            values: {
                                let mut values = Vec::new();
                                for _ in 0..instance_count {
                                    let mut val_buf = [0u8; size_of::<i16>()];
                                    cursor.read_exact(&mut val_buf)?;
                                    let val = i16::from_be_bytes(val_buf);
                                    values.push(val);
                                }
                                values
                            },
                        },
                        4 => AttrValue::U32 {
                            values: {
                                let mut values = Vec::new();
                                for _ in 0..instance_count {
                                    let mut val_buf = [0u8; size_of::<u32>()];
                                    cursor.read_exact(&mut val_buf)?;
                                    let val = u32::from_be_bytes(val_buf);
                                    values.push(val);
                                }
                                values
                            },
                        },
                        5 => AttrValue::S32 {
                            values: {
                                let mut values = Vec::new();
                                for _ in 0..instance_count {
                                    let mut val_buf = [0u8; size_of::<i32>()];
                                    cursor.read_exact(&mut val_buf)?;
                                    let val = i32::from_be_bytes(val_buf);
                                    values.push(val);
                                }
                                values
                            },
                        },
                        6 => AttrValue::U64 {
                            values: {
                                let mut values = Vec::new();
                                for _ in 0..instance_count {
                                    let mut val_buf = [0u8; size_of::<u64>()];
                                    cursor.read_exact(&mut val_buf)?;
                                    let val = u64::from_be_bytes(val_buf);
                                    values.push(val);
                                }
                                values
                            },
                        },
                        7 => AttrValue::S64 {
                            values: {
                                let mut values = Vec::new();
                                for _ in 0..instance_count {
                                    let mut val_buf = [0u8; size_of::<i64>()];
                                    cursor.read_exact(&mut val_buf)?;
                                    let val = i64::from_be_bytes(val_buf);
                                    values.push(val);
                                }
                                values
                            },
                        },
                        _ => unreachable!(),
                    };

                    attributes.push(Attribute { id, val })
                }
                Ok(GpuMetrics::F1C9(C9 { attributes }))
            }
        }
    }

    pub mod f2 {
        use super::*;

        // === gpu_metrics_v2_0 ===

        #[repr(C)]
        #[derive(Debug, Clone)]
        pub struct C0 {
            common_header: Header,

            /* Driver attached timestamp (in ns) */
            pub system_clock_counter: u64,

            /* Temperature */
            pub temperature_gfx: u16,       // gfx temperature on APUs
            pub temperature_soc: u16,       // soc temperature on APUs
            pub temperature_core: [u16; 8], // CPU core temperature on APUs
            pub temperature_l3: [u16; 2],

            /* Utilization */
            pub average_gfx_activity: u16,
            pub average_mm_activity: u16, // UVD or VCN

            /* Power/Energy */
            pub average_socket_power: u16, // dGPU + APU power on A + A platform
            pub average_cpu_power: u16,
            pub average_soc_power: u16,
            pub average_gfx_power: u16,
            pub average_core_power: [u16; 8], // CPU core power on APUs

            /* Average clocks */
            pub average_gfxclk_frequency: u16,
            pub average_socclk_frequency: u16,
            pub average_uclk_frequency: u16,
            pub average_fclk_frequency: u16,
            pub average_vclk_frequency: u16,
            pub average_dclk_frequency: u16,

            /* Current clocks */
            pub current_gfxclk: u16,
            pub current_socclk: u16,
            pub current_uclk: u16,
            pub current_fclk: u16,
            pub current_vclk: u16,
            pub current_dclk: u16,
            pub current_coreclk: [u16; 8], // CPU core clocks
            pub current_l3clk: [u16; 2],

            /* Throttle status */
            pub throttle_status: u32,

            /* Fans */
            pub fan_pwm: u16,

            padding: u16,
        }

        impl AmdgpuMetrics for C0 {
            fn from_bytes(bytes: &[u8]) -> anyhow::Result<GpuMetrics> {
                tracing::debug!("Reading format 2 content 0");
                let bytes = from_bytes::<Self>(bytes);
                Ok(GpuMetrics::F2C0(bytes.ok_or_else(|| {
                    anyhow::anyhow!("gpu_metrics file struct malformed")
                })?))
            }
        }

        // === gpu_metrics_v2_1 ===

        #[repr(C)]
        #[derive(Debug, Clone)]
        pub struct C1 {
            common_header: Header,

            /* Temperature */
            pub temperature_gfx: u16,       // gfx temperature on APUs
            pub temperature_soc: u16,       // soc temperature on APUs
            pub temperature_core: [u16; 8], // CPU core temperature on APUs
            pub temperature_l3: [u16; 2],

            /* Utilization */
            pub average_gfx_activity: u16,
            pub average_mm_activity: u16, // UVD or VCN

            /* Driver attached timestamp (in ns) */
            pub system_clock_counter: u64,

            /* Power/Energy */
            pub average_socket_power: u16, // dGPU + APU power on A + A platform
            pub average_cpu_power: u16,
            pub average_soc_power: u16,
            pub average_gfx_power: u16,
            pub average_core_power: [u16; 8], // CPU core power on APUs

            /* Average clocks */
            pub average_gfxclk_frequency: u16,
            pub average_socclk_frequency: u16,
            pub average_uclk_frequency: u16,
            pub average_fclk_frequency: u16,
            pub average_vclk_frequency: u16,
            pub average_dclk_frequency: u16,

            /* Current clocks */
            pub current_gfxclk: u16,
            pub current_socclk: u16,
            pub current_uclk: u16,
            pub current_fclk: u16,
            pub current_vclk: u16,
            pub current_dclk: u16,
            pub current_coreclk: [u16; 8], // CPU core clocks
            pub current_l3clk: [u16; 2],

            /* Throttle status */
            pub throttle_status: u32,

            /* Fans */
            pub fan_pwm: u16,

            padding: [u16; 3],
        }

        impl AmdgpuMetrics for C1 {
            fn from_bytes(bytes: &[u8]) -> anyhow::Result<GpuMetrics> {
                tracing::debug!("Reading format 2 content 1");
                let bytes = from_bytes::<Self>(bytes);
                Ok(GpuMetrics::F2C1(bytes.ok_or_else(|| {
                    anyhow::anyhow!("gpu_metrics file struct malformed")
                })?))
            }
        }

        // === gpu_metrics_v2_2 ===

        #[repr(C)]
        #[derive(Debug, Clone)]
        pub struct C2 {
            common_header: Header,

            /* Temperature */
            pub temperature_gfx: u16,       // gfx temperature on APUs
            pub temperature_soc: u16,       // soc temperature on APUs
            pub temperature_core: [u16; 8], // CPU core temperature on APUs
            pub temperature_l3: [u16; 2],

            /* Utilization */
            pub average_gfx_activity: u16,
            pub average_mm_activity: u16, // UVD or VCN

            /* Driver attached timestamp (in ns) */
            pub system_clock_counter: u64,

            /* Power/Energy */
            pub average_socket_power: u16, // dGPU + APU power on A + A platform
            pub average_cpu_power: u16,
            pub average_soc_power: u16,
            pub average_gfx_power: u16,
            pub average_core_power: [u16; 8], // CPU core power on APUs

            /* Average clocks */
            pub average_gfxclk_frequency: u16,
            pub average_socclk_frequency: u16,
            pub average_uclk_frequency: u16,
            pub average_fclk_frequency: u16,
            pub average_vclk_frequency: u16,
            pub average_dclk_frequency: u16,

            /* Current clocks */
            pub current_gfxclk: u16,
            pub current_socclk: u16,
            pub current_uclk: u16,
            pub current_fclk: u16,
            pub current_vclk: u16,
            pub current_dclk: u16,
            pub current_coreclk: [u16; 8], // CPU core clocks
            pub current_l3clk: [u16; 2],

            /* Throttle status (ASIC dependent) */
            pub throttle_status: u32,

            /* Fans */
            pub fan_pwm: u16,

            padding: [u16; 3],

            /* Throttle status (ASIC independent) */
            pub indep_throttle_status: u64,
        }

        impl AmdgpuMetrics for C2 {
            fn from_bytes(bytes: &[u8]) -> anyhow::Result<GpuMetrics> {
                tracing::debug!("Reading format 2 content 2");
                let bytes = from_bytes::<Self>(bytes);
                Ok(GpuMetrics::F2C2(bytes.ok_or_else(|| {
                    anyhow::anyhow!("gpu_metrics file struct malformed")
                })?))
            }
        }

        // === gpu_metrics_v2_3 ===

        #[repr(C)]
        #[derive(Debug, Clone)]
        pub struct C3 {
            common_header: Header,

            /* Temperature */
            pub temperature_gfx: u16,       // gfx temperature on APUs
            pub temperature_soc: u16,       // soc temperature on APUs
            pub temperature_core: [u16; 8], // CPU core temperature on APUs
            pub temperature_l3: [u16; 2],

            /* Utilization */
            pub average_gfx_activity: u16,
            pub average_mm_activity: u16, // UVD or VCN

            /* Driver attached timestamp (in ns) */
            pub system_clock_counter: u64,

            /* Power/Energy */
            pub average_socket_power: u16, // dGPU + APU power on A + A platform
            pub average_cpu_power: u16,
            pub average_soc_power: u16,
            pub average_gfx_power: u16,
            pub average_core_power: [u16; 8], // CPU core power on APUs

            /* Average clocks */
            pub average_gfxclk_frequency: u16,
            pub average_socclk_frequency: u16,
            pub average_uclk_frequency: u16,
            pub average_fclk_frequency: u16,
            pub average_vclk_frequency: u16,
            pub average_dclk_frequency: u16,

            /* Current clocks */
            pub current_gfxclk: u16,
            pub current_socclk: u16,
            pub current_uclk: u16,
            pub current_fclk: u16,
            pub current_vclk: u16,
            pub current_dclk: u16,
            pub current_coreclk: [u16; 8], // CPU core clocks
            pub current_l3clk: [u16; 2],

            /* Throttle status (ASIC dependent) */
            pub throttle_status: u32,

            /* Fans */
            pub fan_pwm: u16,

            padding: [u16; 3],

            /* Throttle status (ASIC independent) */
            pub indep_throttle_status: u64,

            /* Average Temperature */
            pub average_temperature_gfx: u16, // average gfx temperature on APUs
            pub average_temperature_soc: u16, // average soc temperature on APUs
            pub average_temperature_core: [u16; 8], // average CPU core temperature on APUs
            pub average_temperature_l3: [u16; 2],
        }

        impl AmdgpuMetrics for C3 {
            fn from_bytes(bytes: &[u8]) -> anyhow::Result<GpuMetrics> {
                tracing::debug!("Reading format 2 content 3");
                let bytes = from_bytes::<Self>(bytes);
                Ok(GpuMetrics::F2C3(bytes.ok_or_else(|| {
                    anyhow::anyhow!("gpu_metrics file struct malformed")
                })?))
            }
        }

        // === gpu_metrics_v2_4 ===

        #[repr(C)]
        #[derive(Debug, Clone)]
        pub struct C4 {
            common_header: Header,

            /* Temperature (unit: centi-Celsius) */
            pub temperature_gfx: u16,
            pub temperature_soc: u16,
            pub temperature_core: [u16; 8],
            pub temperature_l3: [u16; 2],

            /* Utilization (unit: centi) */
            pub average_gfx_activity: u16,
            pub average_mm_activity: u16,

            /* Driver attached timestamp (in ns) */
            pub system_clock_counter: u64,

            /* Power/Energy (unit: mW) */
            pub average_socket_power: u16,
            pub average_cpu_power: u16,
            pub average_soc_power: u16,
            pub average_gfx_power: u16,
            pub average_core_power: [u16; 8],

            /* Average clocks (unit: MHz) */
            pub average_gfxclk_frequency: u16,
            pub average_socclk_frequency: u16,
            pub average_uclk_frequency: u16,
            pub average_fclk_frequency: u16,
            pub average_vclk_frequency: u16,
            pub average_dclk_frequency: u16,

            /* Current clocks (unit: MHz) */
            pub current_gfxclk: u16,
            pub current_socclk: u16,
            pub current_uclk: u16,
            pub current_fclk: u16,
            pub current_vclk: u16,
            pub current_dclk: u16,
            pub current_coreclk: [u16; 8],
            pub current_l3clk: [u16; 2],

            /* Throttle status (ASIC dependent) */
            pub throttle_status: u32,

            /* Fans */
            pub fan_pwm: u16,

            padding: [u16; 3],

            /* Throttle status (ASIC independent) */
            pub indep_throttle_status: u64,

            /* Average Temperature (unit: centi-Celsius) */
            pub average_temperature_gfx: u16,
            pub average_temperature_soc: u16,
            pub average_temperature_core: [u16; 8],
            pub average_temperature_l3: [u16; 2],

            /* Power/Voltage (unit: mV) */
            pub average_cpu_voltage: u16,
            pub average_soc_voltage: u16,
            pub average_gfx_voltage: u16,

            /* Power/Current (unit: mA) */
            pub average_cpu_current: u16,
            pub average_soc_current: u16,
            pub average_gfx_current: u16,
        }

        impl AmdgpuMetrics for C4 {
            fn from_bytes(bytes: &[u8]) -> anyhow::Result<GpuMetrics> {
                tracing::debug!("Reading format 2 content 4");
                let bytes = from_bytes::<Self>(bytes);
                Ok(GpuMetrics::F2C4(bytes.ok_or_else(|| {
                    anyhow::anyhow!("gpu_metrics file struct malformed")
                })?))
            }
        }
    }

    pub mod f3 {
        use super::*;

        // === gpu_metrics_v3_0 ===

        #[repr(C)]
        #[derive(Debug, Clone)]
        pub struct C0 {
            common_header: Header,

            /* Temperature */
            /* gfx temperature on APUs */
            pub temperature_gfx: u16,
            /* soc temperature on APUs */
            pub temperature_soc: u16,
            /* CPU core temperature on APUs */
            pub temperature_core: [u16; 16],
            /* skin temperature on APUs */
            pub temperature_skin: u16,

            /* Utilization */
            /* time filtered GFX busy % [0-100] */
            pub average_gfx_activity: u16,
            /* time filtered VCN busy % [0-100] */
            pub average_vcn_activity: u16,
            /* time filtered IPU per-column busy % [0-100] */
            pub average_ipu_activity: [u16; 8],
            /* time filtered per-core C0 residency % [0-100]*/
            pub average_core_c0_activity: [u16; 16],
            /* time filtered DRAM read bandwidth [MB/sec] */
            pub average_dram_reads: u16,
            /* time filtered DRAM write bandwidth [MB/sec] */
            pub average_dram_writes: u16,
            /* time filtered IPU read bandwidth [MB/sec] */
            pub average_ipu_reads: u16,
            /* time filtered IPU write bandwidth [MB/sec] */
            pub average_ipu_writes: u16,

            /* Driver attached timestamp (in ns) */
            pub system_clock_counter: u64,

            /* Power/Energy */
            /* time filtered power used for PPT/STAPM [APU+dGPU] [mW] */
            pub average_socket_power: u32,
            /* time filtered IPU power [mW] */
            pub average_ipu_power: u16,
            /* time filtered APU power [mW] */
            pub average_apu_power: u32,
            /* time filtered GFX power [mW] */
            pub average_gfx_power: u32,
            /* time filtered dGPU power [mW] */
            pub average_dgpu_power: u32,
            /* time filtered sum of core power across all cores in the socket [mW] */
            pub average_all_core_power: u32,
            /* calculated core power [mW] */
            pub average_core_power: [u16; 16],
            /* time filtered total system power [mW] */
            pub average_sys_power: u16,
            /* maximum IRM defined STAPM power limit [mW] */
            pub stapm_power_limit: u16,
            /* time filtered STAPM power limit [mW] */
            pub current_stapm_power_limit: u16,

            /* time filtered clocks [MHz] */
            pub average_gfxclk_frequency: u16,
            pub average_socclk_frequency: u16,
            pub average_vpeclk_frequency: u16,
            pub average_ipuclk_frequency: u16,
            pub average_fclk_frequency: u16,
            pub average_vclk_frequency: u16,
            pub average_uclk_frequency: u16,
            pub average_mpipu_frequency: u16,

            /* Current clocks */
            /* target core frequency [MHz] */
            pub current_coreclk: [u16; 16],
            /* CCLK frequency limit enforced on classic cores [MHz] */
            pub current_core_maxfreq: u16,
            /* GFXCLK frequency limit enforced on GFX [MHz] */
            pub current_gfx_maxfreq: u16,

            /* Throttle Residency (ASIC dependent) */
            pub throttle_residency_prochot: u32,
            pub throttle_residency_spl: u32,
            pub throttle_residency_fppt: u32,
            pub throttle_residency_sppt: u32,
            pub throttle_residency_thm_core: u32,
            pub throttle_residency_thm_gfx: u32,
            pub throttle_residency_thm_soc: u32,

            /* Metrics table alpha filter time constant [us] */
            pub time_filter_alphavalue: u32,
        }

        impl AmdgpuMetrics for C0 {
            fn from_bytes(bytes: &[u8]) -> anyhow::Result<GpuMetrics> {
                tracing::debug!("Reading format 3 content 0");
                let bytes = from_bytes::<Self>(bytes);
                Ok(GpuMetrics::F3C0(bytes.ok_or_else(|| {
                    anyhow::anyhow!("gpu_metrics file struct malformed")
                })?))
            }
        }
    }
}
