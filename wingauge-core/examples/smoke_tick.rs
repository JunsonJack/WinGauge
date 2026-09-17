//! 采集冒烟：跑一帧快照并打印，验证 sysinfo 通路在本机真实可读。
//! 用法：`cargo run -p wingauge-core --example smoke_tick`

use wingauge_core::Collector;

fn main() {
    println!("== WinGauge collector smoke ==");
    let mut c = Collector::new();
    println!("priming…");
    c.prime();
    println!("primed = {}", c.is_primed());

    // 第二帧才有网络速率
    let _ = c.tick();
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let snap = c.tick();

    match &snap.device {
        Some(d) => println!(
            "device  : host={} os={} {} cores={} mem={:.1}GB uptime={}s",
            d.host,
            d.os,
            d.os_version,
            d.logical_cores,
            d.total_memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
            d.uptime_secs
        ),
        None => println!("device  : None"),
    }

    match &snap.cpu {
        Some(cpu) => {
            println!(
                "cpu     : {:.1}% peak={:?} temp={:?}°C cores={}",
                cpu.usage, cpu.peak, cpu.temp_c, cpu.per_core.len()
            );
            let sample: Vec<String> = cpu.per_core.iter().take(8).map(|v| format!("{v:.0}")).collect();
            println!("          per-core[:8] = {:?}", sample);
        }
        None => println!("cpu     : None"),
    }

    match &snap.thermal {
        Some(t) => {
            println!(
                "thermal : cpu={:?} gpu={:?} fans={:?} other={}",
                t.cpu_temp_c,
                t.gpu_temp_c,
                t.fans,
                t.other_sensors.len()
            );
        }
        None => println!("thermal : None（非联想或未命中）"),
    }

    match &snap.memory {
        Some(m) => println!(
            "memory  : {:.1}% used={:.2}/{:.2}GB committed={:?}",
            m.usage,
            m.used_bytes as f64 / 1e9,
            m.total_bytes as f64 / 1e9,
            m.committed_bytes.map(|c| format!("{:.2}GB", c as f64 / 1e9))
        ),
        None => println!("memory  : None"),
    }

    match &snap.network {
        Some(n) => println!(
            "network : {} down={}/s up={}/s",
            n.friendly_name, n.download_bps, n.upload_bps
        ),
        None => println!("network : None"),
    }

    match &snap.disk {
        Some(d) => {
            let s = &d.system_drive;
            println!(
                "disk    : {} used={:.1}% {:.1}/{:.1}GB",
                s.letter,
                s.usage(),
                s.used_bytes as f64 / 1e9,
                s.total_bytes as f64 / 1e9
            );
        }
        None => println!("disk    : None"),
    }

    match &snap.health {
        Some(h) => {
            println!("health  : {} ({:?}) — {}", h.score, h.band, h.summary);
            for i in &h.issues {
                println!("          -{} {}", i.points, i.reason);
            }
        }
        None => println!("health  : None"),
    }

    let ok = snap.cpu.is_some() && snap.memory.is_some() && snap.disk.is_some() && snap.health.is_some();
    if ok {
        println!("\nSMOKE OK");
    } else {
        eprintln!("\nSMOKE FAIL: some cards missing");
        std::process::exit(1);
    }
}
