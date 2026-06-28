/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::collections::HashMap;

use rustix::fd::AsFd;
use rustix::fs::{Mode, OFlags};

use super::helpers::*;

#[doc(inline)]
pub use crate::metrics::process::*;

pub struct Collector {
    cpu_counters: HashMap<PidId, CpuCounters>,
    prev_gpu_fdinfo: HashMap<u32, DrmFdinfo>,
    disk_counters: HashMap<PidId, DiskCounters>,
    net_counters: HashMap<PidId, HashMap<String, NetUsage>>,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector {
    pub fn new() -> Self {
        tracing::info!("creating collector");
        Self {
            cpu_counters: HashMap::new(),
            prev_gpu_fdinfo: HashMap::new(),
            disk_counters: HashMap::new(),
            net_counters: HashMap::new(),
        }
    }
}

impl super::Collector for Collector {
    type Output = Snapshot;

    fn name(&self) -> &'static str {
        "process"
    }

    fn collect(&mut self, config: &crate::metrics::Config) -> anyhow::Result<Self::Output> {
        let Some(config) = config.process.as_ref() else {
            return Ok(Snapshot::default());
        };
        let mut snapshot = Snapshot::default();

        let mut cpu_counters = HashMap::new();
        let mut cur_gpu_fdinfo = HashMap::new();
        let mut disk_counters = HashMap::new();
        let mut net_counters: HashMap<PidId, HashMap<String, NetUsage>> = HashMap::new();

        for proc in procfs::process::all_processes()?.flatten() {
            let Ok(stat) = proc.stat() else {
                continue;
            };
            // Skip kernel threads
            if stat.flags & 0x00200000 != 0 {
                continue;
            }
            let Ok(status) = proc.status() else {
                continue;
            };

            let pid_id = PidId {
                pid: proc.pid as u32,
                timestamp: stat.starttime,
            };

            let mut usage: Option<Usage> = None;

            if config.cpu_usage {
                let usage = usage.get_or_insert_default();

                let cur = CpuCounters {
                    utime: stat.utime,
                    stime: stat.stime,
                };

                if let Some(prev) = self.cpu_counters.get_mut(&pid_id) {
                    let util = ((cur.utime - prev.utime) + (cur.stime - prev.stime)) as f64
                        / procfs::ticks_per_second() as f64
                        * 100.0;
                    let mut affinity = Vec::new();
                    if let Some(allowed) = status.cpus_allowed_list {
                        for range in allowed {
                            for i in range.0..=range.1 {
                                affinity.push(i)
                            }
                        }
                    }
                    usage.cpu = Some(CpuUsage {
                        usage: util as u32,
                        threads: stat.num_threads as u32,
                        nice: stat.nice as i32,
                        affinity,
                    });
                }
                cpu_counters.insert(pid_id, cur);
            }
            if config.memory_usage {
                let usage = usage.get_or_insert_with(|| Usage::default());

                if let Ok(statm) = proc.statm() {
                    usage.memory = Some(MemoryUsage {
                        usage: statm.resident - statm.shared,
                        resident: statm.resident,
                        shared: statm.shared,
                        r#virtual: statm.size,
                    })
                }
            }

            if config.gpu_usage {
                if let Ok(fdinfo) = proc.fd() {
                    for fd in fdinfo.flatten() {
                        let pid_id = PidId {
                            pid: proc.pid as u32,
                            timestamp: stat.starttime,
                        };

                        if let Ok(cur) = parse_fdinfo(pid_id, fd.fd as u32)
                            && cur.driver.is_some()
                            && let Some(client_id) = cur.client_id
                            && !cur_gpu_fdinfo.contains_key(&client_id)
                        {
                            cur_gpu_fdinfo
                                .entry(client_id)
                                .or_insert(cur)
                                .pids
                                .push(proc.pid as u32);
                        }
                    }
                }
            }

            if config.disk_usage {
                let usage = usage.get_or_insert_default();

                if let Ok(io) = proc.io() {
                    let cur = DiskCounters {
                        read_bytes: io.read_bytes,
                        write_bytes: io.write_bytes,
                    };

                    if let Some(prev) = self.disk_counters.get_mut(&pid_id) {
                        usage.disk = Some(DiskUsage {
                            read_bytes: cur.read_bytes - prev.read_bytes,
                            read_total: cur.read_bytes,
                            write_bytes: cur.write_bytes - prev.write_bytes,
                            write_total: cur.write_bytes,
                        })
                    }
                    disk_counters.insert(pid_id, cur);
                };
            }

            if config.net_usage {
                let usage = usage.get_or_insert_default();

                if let Ok(dev_status) = proc.dev_status() {
                    let proc_prev = self.net_counters.entry(pid_id).or_default();
                    for (dev, status) in dev_status {
                        // filter out non-real network devices
                        if dev == "lo"
                            || dev.starts_with("veth")
                            || dev.starts_with("docker")
                            || dev.starts_with("br-")
                            || dev.starts_with("cni")
                            || dev.starts_with("flannel")
                            || dev.starts_with("cali")
                            || dev.starts_with("virbr")
                            || dev.starts_with("vnet")
                            || dev.starts_with("vmnet")
                            || dev.starts_with("vboxnet")
                            || dev.starts_with("tun")
                            || dev.starts_with("tap")
                            || dev.starts_with("wg")
                            || dev.starts_with("sit")
                            || dev.starts_with("ipip")
                            || dev.starts_with("dummy")
                            || dev.starts_with("ifb")
                            || dev.starts_with("teql")
                        {
                            continue;
                        }
                        let cur = NetUsage {
                            recv_bytes: status.recv_bytes,
                            recv_packets: status.recv_packets,
                            recv_errors: status.recv_errs,
                            recv_drop: status.recv_drop,
                            send_bytes: status.sent_bytes,
                            send_packets: status.sent_packets,
                            send_errors: status.sent_errs,
                            send_drop: status.sent_drop,
                        };
                        if let Some(prev) = proc_prev.get(&dev) {
                            usage.net.insert(
                                dev.clone(),
                                NetUsage {
                                    recv_bytes: cur.recv_bytes - prev.recv_bytes,
                                    recv_packets: cur.recv_packets - prev.recv_packets,
                                    recv_errors: cur.recv_errors - prev.recv_errors,
                                    recv_drop: cur.recv_drop - prev.recv_drop,
                                    send_bytes: cur.send_bytes - prev.send_bytes,
                                    send_packets: cur.send_packets - prev.send_packets,
                                    send_errors: cur.send_errors - prev.send_errors,
                                    send_drop: cur.send_drop - prev.send_drop,
                                },
                            );
                        }
                        net_counters
                            .entry(pid_id)
                            .or_default()
                            .insert(dev.clone(), cur);
                    }
                }
            }

            snapshot.processes.insert(
                proc.pid as u32,
                Process {
                    identity: config.identity.then(|| Identity {
                        pid: proc.pid as u32,
                        ppid: stat.ppid as u32,
                        uid: proc.uid().unwrap_or(0),
                        gid: status.egid,
                        session: stat.session,
                        name: stat.comm.clone(),
                        exe: proc
                            .exe()
                            .map(|e| e.to_string_lossy().into_owned())
                            .unwrap_or_default(),
                        cmdline: proc
                            .cmdline()
                            .map(|c| c.into_iter().collect::<Vec<_>>().join(" "))
                            .unwrap_or_default(),
                    }),
                    status: config
                        .status
                        .then(|| {
                            // test
                            use procfs::process::ProcState;
                            stat.state()
                                .map(|s| match s {
                                    ProcState::Running => Status::Running as i32,
                                    ProcState::Sleeping => Status::Sleeping as i32,
                                    ProcState::Waiting => Status::DiskSleep as i32,
                                    ProcState::Stopped => Status::Stopped as i32,
                                    ProcState::Tracing => Status::Tracing as i32,
                                    ProcState::Zombie => Status::Zombie as i32,
                                    ProcState::Idle => Status::Idle as i32,
                                    ProcState::Wakekill => Status::WakeKill as i32,
                                    ProcState::Waking => Status::Waking as i32,
                                    ProcState::Parked => Status::Parked as i32,
                                    ProcState::Dead => Status::Dead as i32,
                                })
                                .ok()
                        })
                        .flatten()
                        .unwrap_or(-1),
                    start_time: config
                        .start_time
                        .then(|| stat.starttime)
                        .unwrap_or_default(),
                    usage,
                },
            );
        }

        // Iterate over fdinfos and calculate GPU usage, as well as oldest timestamp (the progenitor of the fd)
        for (client_id, cur) in cur_gpu_fdinfo.iter() {
            if let Some(prev) = self.prev_gpu_fdinfo.get(client_id)
                && let Some(pdev) = cur.pdev.clone()
            {
                let Some(fd_usage) = diff_fdinfo(prev, cur) else {
                    continue;
                };
                if let Some(oldest) = cur.pids.iter().min_by(|pid_a, pid_b| {
                    snapshot.processes[pid_a]
                        .start_time
                        .cmp(&snapshot.processes[pid_b].start_time)
                }) {
                    let Some(proc) = snapshot.processes.get_mut(oldest) else {
                        continue;
                    };
                    let usage = proc.usage.get_or_insert_default();
                    merge_gpu_usage(usage.gpu.entry(pdev.clone()).or_default(), &fd_usage);
                }
            }
        }

        self.cpu_counters = cpu_counters;
        self.prev_gpu_fdinfo = cur_gpu_fdinfo;
        self.disk_counters = disk_counters;
        self.net_counters = net_counters;

        Ok(snapshot)
    }
}

impl super::Resolver for Collector {
    type Input = crate::metrics::gpu::Snapshot;

    fn resolve(&mut self, input: &Self::Input, output: &mut Self::Output) -> anyhow::Result<()> {
        for device in input.gpus.iter() {
            for gpu_process in device.processes.iter() {
                if let Some(process) = output.processes.get_mut(&gpu_process.pid) {
                    let process_usage = process.usage.get_or_insert_default();
                    let process_gpu_usage =
                        process_usage.gpu.entry(device.pci_id.clone()).or_default();
                    for engine in gpu_process.engine_utilization.iter() {
                        let Some(identifier) = engine.identifier.as_ref() else {
                            continue;
                        };
                        process_gpu_usage.engines.insert(
                            match identifier.r#type {
                                0 => "unspecified".to_string(),
                                1 => "graphics".to_string(),
                                2 => "compute".to_string(),
                                3 => "copy".to_string(),
                                4 => "memory_controller".to_string(),
                                5 => "video_decode".to_string(),
                                6 => "video_encode".to_string(),
                                7 => "video_unified".to_string(),
                                8 => "jpeg".to_string(),
                                9 => "media_clear".to_string(),
                                _ => "other".to_string(),
                            },
                            engine.utilization as u32,
                        );
                    }
                    process_gpu_usage.system_usage = gpu_process.gtt_usage;
                    process_gpu_usage.vram_usage = gpu_process.vram_usage;
                }
            }
        }
        Ok(())
    }
}

struct CpuCounters {
    utime: u64,
    stime: u64,
}

struct DiskCounters {
    read_bytes: u64,
    write_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PidId {
    pid: u32,
    timestamp: u64,
}

#[derive(Debug, Clone)]
struct DrmFdinfo {
    timestamp: std::time::Instant,
    driver: Option<String>,
    client_id: Option<u32>,
    pdev: Option<String>,
    times: HashMap<String, u64>,
    cycles: HashMap<String, u64>,
    total_cycles: HashMap<String, u64>,
    maxfreq: HashMap<String, u64>,
    shared_mem: HashMap<String, u64>,
    resident_mem: HashMap<String, u64>,

    pids: Vec<u32>,
}

impl Default for DrmFdinfo {
    fn default() -> Self {
        Self {
            timestamp: std::time::Instant::now(),
            driver: None,
            client_id: None,
            pdev: None,
            times: HashMap::new(),
            cycles: HashMap::new(),
            total_cycles: HashMap::new(),
            maxfreq: HashMap::new(),
            shared_mem: HashMap::new(),
            resident_mem: HashMap::new(),
            pids: Vec::new(),
        }
    }
}

fn parse_fdinfo(proc: PidId, fd: u32) -> anyhow::Result<DrmFdinfo> {
    // open the fdinfo file. we can safely assume that the pid is not reused because the collect function still has an open pidfd.
    let path = format!("/proc/{}/fdinfo/{}", proc.pid, fd);
    let file = rustix::fs::open(path, OFlags::RDONLY | OFlags::CLOEXEC, Mode::empty())?;

    let contents = sysfs::read_string(file.as_fd())
        .ok_or_else(|| anyhow::anyhow!("failed to read fdinfo file"))?;

    let mut fdinfo = DrmFdinfo::default();
    fdinfo.pids.push(proc.pid);

    // parse the fdinfo file
    for line in contents.lines() {
        if let Some((key, value)) = line.split_once(':') {
            if key == "drm-driver" {
                fdinfo.driver = Some(value.trim().to_string());
            } else if key == "drm-client-id" {
                fdinfo.client_id = value.trim().parse().ok();
            } else if key == "drm-pdev" {
                fdinfo.pdev = Some(value.trim().to_string());
            } else if key.starts_with("drm-engine-") {
                if let Some(engine) = key.strip_prefix("drm-engine-") {
                    fdinfo.times.insert(
                        engine.to_string(),
                        value
                            .trim()
                            .strip_suffix(" ms")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0),
                    );
                }
            } else if key.starts_with("drm-cycles-") {
                if let Some(engine) = key.strip_prefix("drm-cycles-") {
                    fdinfo
                        .cycles
                        .insert(engine.to_string(), value.trim().parse().unwrap_or(0));
                }
            } else if key.starts_with("drm-total-cycles-") {
                if let Some(engine) = key.strip_prefix("drm-total-cycles-") {
                    fdinfo
                        .total_cycles
                        .insert(engine.to_string(), value.trim().parse().unwrap_or(0));
                }
            } else if key.starts_with("drm-maxfreq-") {
                if let Some(engine) = key.strip_prefix("drm-maxfreq-") {
                    let (freq, unit) = value.trim().split_once(' ').unwrap_or((value.trim(), "Hz"));
                    match unit {
                        "Hz" => {
                            fdinfo
                                .maxfreq
                                .insert(engine.to_string(), freq.parse().unwrap_or(0));
                        }
                        "KHz" => {
                            fdinfo
                                .maxfreq
                                .insert(engine.to_string(), freq.parse().unwrap_or(0) * 1000);
                        }
                        "MHz" => {
                            fdinfo.maxfreq.insert(
                                engine.to_string(),
                                freq.parse().unwrap_or(0) * 1000 * 1000,
                            );
                        }
                        _ => {}
                    }
                }
            } else if key.starts_with("drm-shared-") {
                if let Some(engine) = key.strip_prefix("drm-shared-") {
                    let (bytes, unit) = value
                        .trim()
                        .split_once(' ')
                        .unwrap_or((value.trim(), "KiB"));
                    match unit {
                        "KiB" => {
                            fdinfo
                                .shared_mem
                                .insert(engine.to_string(), bytes.parse().unwrap_or(0) * 1024);
                        }
                        "MiB" => {
                            fdinfo.shared_mem.insert(
                                engine.to_string(),
                                bytes.parse().unwrap_or(0) * 1024 * 1024,
                            );
                        }
                        _ => {}
                    }
                }
            } else if key.starts_with("drm-resident-") {
                if let Some(engine) = key.strip_prefix("drm-resident-") {
                    let (bytes, unit) = value
                        .trim()
                        .split_once(' ')
                        .unwrap_or((value.trim(), "KiB"));
                    match unit {
                        "KiB" => {
                            fdinfo
                                .resident_mem
                                .insert(engine.to_string(), bytes.parse().unwrap_or(0) * 1024);
                        }
                        "MiB" => {
                            fdinfo.resident_mem.insert(
                                engine.to_string(),
                                bytes.parse().unwrap_or(0) * 1024 * 1024,
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(fdinfo)
}

fn diff_fdinfo(prev: &DrmFdinfo, cur: &DrmFdinfo) -> Option<GpuUsage> {
    let mut result = GpuUsage::default();
    for (region, &cur_shared) in cur.shared_mem.iter() {
        let Some(&cur_resident) = cur.resident_mem.get(region) else {
            continue;
        };
        if region.starts_with("vram") {
            result.vram_usage += cur_resident - cur_shared;
        } else if region.contains("system")
            || region.contains("cpu")
            || region == "gtt"
            || region == "memory"
        {
            result.system_usage += cur_resident - cur_shared;
        }
    }
    if !cur.cycles.is_empty() {
        for (engine, &cur_cycles) in cur.cycles.iter() {
            let Some(&prev_cycles) = prev.cycles.get(engine) else {
                continue;
            };

            let cycle_diff = cur_cycles.saturating_sub(prev_cycles);

            // priority to use total cycles since that's more of a "utilization" metric
            if let Some(&cur_total_cycles) = cur.total_cycles.get(engine)
                && let Some(&prev_total_cycles) = prev.total_cycles.get(engine)
                && cycle_diff > 0
            {
                let total_cycle_diff = cur_total_cycles.saturating_sub(prev_total_cycles);
                if total_cycle_diff > 0 {
                    result
                        .engines
                        .insert(engine.clone(), (total_cycle_diff * 100 / cycle_diff) as u32);
                }
            } else if let Some(&max_freq) = cur.maxfreq.get(engine) {
                if max_freq > 0 {
                    result
                        .engines
                        .insert(engine.clone(), (cycle_diff * 100 / max_freq) as u32);
                }
            }
        }
    } else if !cur.times.is_empty() {
        for (engine, &cur_time) in cur.times.iter() {
            let Some(&prev_time) = prev.times.get(engine) else {
                continue;
            };
            let time_diff = cur_time.saturating_sub(prev_time);
            let total_time_diff = cur.timestamp - prev.timestamp;
            if total_time_diff.as_nanos() > 0 {
                result.engines.insert(
                    engine.clone(),
                    (time_diff * 100 / total_time_diff.as_nanos() as u64) as u32,
                );
            }
        }
    } else if result.vram_usage == 0 && result.system_usage == 0 {
        return None;
    }

    Some(result)
}

fn merge_gpu_usage(accumulator: &mut GpuUsage, usage: &GpuUsage) {
    for (engine, utilization) in &usage.engines {
        *accumulator.engines.entry(engine.clone()).or_default() += *utilization;
    }
    accumulator.vram_usage += usage.vram_usage;
    accumulator.system_usage += usage.system_usage;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collector::Collector;
    use crate::collector::Resolver;

    #[tracing_test::traced_test]
    #[test]
    fn process() -> anyhow::Result<()> {
        let mut collector = super::Collector::new();
        let mut config = crate::metrics::Config::default();
        config.process = Some(Config {
            identity: true,
            status: true,
            start_time: true,
            cpu_usage: true,
            memory_usage: true,
            gpu_usage: true,
            disk_usage: true,
            net_usage: true,
        });
        let _ = collector.collect(&config)?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        let snapshot = collector.collect(&config)?;

        println!("{:#?}", snapshot.processes);

        Ok(())
    }

    #[tracing_test::traced_test]
    #[test]
    fn proc_resolve() -> anyhow::Result<()> {
        let mut proc_collector = super::Collector::new();
        let mut gpu_collector = crate::collector::gpu::Collector::new();
        let mut config = crate::metrics::Config::default();
        config.gpu = Some(crate::metrics::gpu::Config {
            drivers: false,
            engines: true,
            clocks: false,
            memory: false,
            power: false,
            thermals: false,
            processes: true,
        });
        config.process = Some(crate::metrics::process::Config {
            identity: true,
            status: true,
            start_time: true,
            cpu_usage: false,
            memory_usage: false,
            gpu_usage: true,
            disk_usage: false,
            net_usage: false,
        });
        let _ = proc_collector.collect(&config)?;
        let _ = gpu_collector.collect(&config)?;
        std::thread::sleep(std::time::Duration::from_secs(1));

        let mut proc = proc_collector.collect(&config)?;
        let mut gpu = gpu_collector.collect(&config)?;
        proc_collector.resolve(&gpu, &mut proc)?;
        gpu_collector.resolve(&proc, &mut gpu)?;

        print_processes_gpu(&proc);

        Ok(())
    }

    fn print_processes_gpu(snapshot: &Snapshot) {
        for process in snapshot.processes.values() {
            if process
                .usage
                .as_ref()
                .is_some_and(|usage| !usage.gpu.is_empty())
            {
                println!(
                    "PID: {}, ({})",
                    process.identity.as_ref().map(|i| i.pid).unwrap_or(0),
                    process
                        .identity
                        .as_ref()
                        .map(|i| i.name.clone())
                        .unwrap_or_default()
                );

                for (device, gpu_usage) in process.usage.as_ref().unwrap().gpu.iter() {
                    println!("  Device: {device}");
                    println!("    VRAM Usage: {}", bytes(gpu_usage.vram_usage));
                    println!("    System Memory Usage: {}", bytes(gpu_usage.system_usage));
                    for (engine, &utilization) in gpu_usage.engines.iter() {
                        println!("    Engine: {} (utilization: {})", engine, utilization);
                    }
                }
                println!("");
            }
        }
    }

    fn bytes(b: u64) -> String {
        if b / 1 < 1024 {
            format!("{b} bytes")
        } else if b / 1024 < 1024 {
            format!("{} KiB", b / 1024)
        } else if b / 1024 / 1024 < 1024 {
            format!("{} MiB", b / 1024 / 1024)
        } else {
            format!("{} GiB", b / 1024 / 1024 / 1024)
        }
    }
}
