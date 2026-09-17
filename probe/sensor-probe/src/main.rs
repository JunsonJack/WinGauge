use std::thread::sleep;
use std::time::{Duration, Instant};

use sysinfo::{Components, Disks, Networks, System};

fn secs_to_human(s: u64) -> String {
    format!("{}d {}h {}m", s / 86400, (s % 86400) / 3600, (s % 3600) / 60)
}

fn main() {
    println!("=== sysinfo {} probe ===", env!("CARGO_PKG_NAME"));

    // ---- 设备头 ----
    println!(
        "[identity] host={:?} os={:?} long_os={:?} kernel={:?} arch={:?}",
        System::host_name(),
        System::os_version(),
        System::long_os_version(),
        System::kernel_version(),
        System::cpu_arch()
    );
    println!(
        "[identity] uptime={} boot_time={}",
        secs_to_human(System::uptime()),
        System::boot_time()
    );

    // ---- CPU：首次构造 + 逐次刷新耗时（决定 1s 采样预算） ----
    let t = Instant::now();
    let mut sys = System::new_all();
    println!("[cpu] System::new_all() 冷启动耗时 {:?}", t.elapsed());

    for i in 0..5 {
        let t = Instant::now();
        sys.refresh_cpu_all();
        let cpu_ms = t.elapsed().as_millis();
        let t2 = Instant::now();
        sys.refresh_memory();
        let mem_ms = t2.elapsed().as_millis();
        println!(
            "[cpu] 第{i}次 refresh_cpu_all={cpu_ms}ms refresh_memory={mem_ms}ms global={:.1}% cores={}",
            sys.global_cpu_usage(),
            sys.cpus().len()
        );
        sleep(Duration::from_millis(1000));
    }
    print!("[cpu] 每核: ");
    for c in sys.cpus() {
        print!("{}={:.0} ", c.name(), c.cpu_usage());
    }
    println!();

    // ---- 内存 / 交换 ----
    println!(
        "[mem] total={:.1}GB used={:.1}GB pct={:.1}% | swap total={:.1}GB used={:.1}GB",
        sys.total_memory() as f64 / 1e9,
        sys.used_memory() as f64 / 1e9,
        sys.used_memory() as f64 / sys.total_memory() as f64 * 100.0,
        sys.total_swap() as f64 / 1e9,
        sys.used_swap() as f64 / 1e9
    );

    // ---- 温度：本轮最关键项 ----
    let t = Instant::now();
    let mut comps = Components::new_with_refreshed_list();
    println!("[thermal] 首次枚举耗时 {:?}，传感器数 = {}", t.elapsed(), comps.len());
    if comps.is_empty() {
        println!("[thermal] ⚠ 一个传感器都没有");
    }
    for c in comps.list() {
        println!(
            "[thermal] label={:?} temp={:?} max={:?} critical={:?}",
            c.label(),
            c.temperature(),
            c.max(),
            c.critical()
        );
    }
    let t = Instant::now();
    comps.refresh(false);
    println!("[thermal] 后续 refresh 耗时 {:?}", t.elapsed());

    // ---- 磁盘 ----
    let t = Instant::now();
    let disks = Disks::new_with_refreshed_list();
    println!("[disk] 枚举耗时 {:?} 数量={}", t.elapsed(), disks.len());
    for d in disks.list() {
        let total = d.total_space();
        let avail = d.available_space();
        println!(
            "[disk] name={:?} fs={:?} mount={:?} removable={} total={:.0}GB used={:.1}%",
            d.name(),
            d.file_system(),
            d.mount_point(),
            d.is_removable(),
            total as f64 / 1e9,
            (1.0 - avail as f64 / total as f64) * 100.0
        );
    }

    // ---- 网络：接口名 + 1 秒速率 ----
    let mut nets = Networks::new_with_refreshed_list();
    println!("[net] 接口数 = {}", nets.len());
    let mut names: Vec<String> = nets.list().iter().map(|(n, _)| n.clone()).collect();
    names.sort();
    println!("[net] 接口清单: {}", names.join(" | "));
    sleep(Duration::from_millis(1200));
    let t = Instant::now();
    nets.refresh(true);
    println!("[net] refresh 耗时 {:?}", t.elapsed());
    let mut top: Vec<(String, u64, u64)> = nets
        .list()
        .iter()
        .map(|(n, d)| (n.clone(), d.received(), d.transmitted()))
        .filter(|(_, r, t)| *r > 0 || *t > 0)
        .collect();
    top.sort_by(|a, b| (b.1 + b.2).cmp(&(a.1 + a.2)));
    if top.is_empty() {
        println!("[net] 1.2s 内所有接口增量均为 0");
    }
    for (n, r, t) in top.iter().take(6) {
        println!("[net] {n}: {:.1} KB/s down, {:.1} KB/s up", *r as f64 / 1e3 / 1.2, *t as f64 / 1e3 / 1.2);
    }

    println!("=== probe end ===");
}
