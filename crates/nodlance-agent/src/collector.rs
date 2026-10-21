//! Host telemetry collector using sysinfo.

use nodlance_domain::HostMetrics;
use sysinfo::{Disks, System};

pub struct MetricsCollector {
    sys: System,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self { sys }
    }

    pub fn collect(&mut self) -> HostMetrics {
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();

        let cpu_percent = self.sys.global_cpu_usage();
        let memory_used_bytes = self.sys.used_memory();
        let memory_total_bytes = self.sys.total_memory();

        let disks = Disks::new_with_refreshed_list();
        let (disk_total, disk_available) = disks.iter().fold((0u64, 0u64), |(tot, avail), disk| {
            (tot + disk.total_space(), avail + disk.available_space())
        });
        let disk_used_bytes = disk_total.saturating_sub(disk_available);

        let load_avg = System::load_average();

        HostMetrics {
            cpu_percent,
            memory_used_bytes,
            memory_total_bytes,
            disk_used_bytes,
            disk_total_bytes: disk_total,
            load_avg_one: load_avg.one as f32,
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
