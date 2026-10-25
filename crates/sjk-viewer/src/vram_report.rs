//! The `vram` console command: the video memory SJK holds, from the graphics
//! allocator's report, summed by what each allocation is labelled (map textures,
//! fonts, saber glows, shadow maps, screen targets...), largest first. The same report
//! a crash on a full card gives ([`crate::gpu_guard`]), on request, so a player can say
//! where a small card's memory goes in the menu, on a map and after a map change. The
//! lines go to the console and the client log.

/// The labels listed; the rest are summed in one line.
const LISTED: usize = 25;
const MIB: f64 = 1024.0 * 1024.0;

/// What `report` says, for the console: totals, then the largest labels.
pub(crate) fn lines(report: Option<&wgpu::AllocatorReport>, card: Option<&str>) -> Vec<String> {
    let mut lines = vec![match card {
        Some(card) => format!("vram: graphics card {card}"),
        None => "vram: graphics card not read on this system".to_owned(),
    }];
    let Some(report) = report else {
        lines.push("vram: this graphics backend gives no allocator report".to_owned());
        return lines;
    };
    lines.push(format!(
        "vram: {:.0} MiB in use, {:.0} MiB reserved in {} blocks",
        report.total_allocated_bytes as f64 / MIB,
        report.total_reserved_bytes as f64 / MIB,
        report.blocks.len()
    ));
    let mut by_label: Vec<(&str, u64, usize)> = Vec::new();
    for allocation in &report.allocations {
        let name = if allocation.name.is_empty() {
            "(unlabelled)"
        } else {
            allocation.name.as_str()
        };
        match by_label.iter_mut().find(|(label, ..)| *label == name) {
            Some((_, bytes, count)) => {
                *bytes += allocation.size;
                *count += 1;
            }
            None => by_label.push((name, allocation.size, 1)),
        }
    }
    by_label.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    for (label, bytes, count) in by_label.iter().take(LISTED) {
        lines.push(format!(
            "vram: {:7.1} MiB  {count:>5} x  {label}",
            *bytes as f64 / MIB
        ));
    }
    let rest = by_label.iter().skip(LISTED);
    let (bytes, labels) = rest.fold((0_u64, 0_usize), |(bytes, labels), entry| {
        (bytes + entry.1, labels + 1)
    });
    if labels > 0 {
        lines.push(format!(
            "vram: {:7.1} MiB  in {labels} other labels",
            bytes as f64 / MIB
        ));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allocation(name: &str, mib: u64) -> wgpu::wgt::AllocationReport {
        wgpu::wgt::AllocationReport {
            name: name.to_owned(),
            offset: 0,
            size: mib << 20,
        }
    }

    #[test]
    fn labels_are_summed_and_listed_largest_first() {
        let report = wgpu::AllocatorReport {
            allocations: vec![
                allocation("fonts", 32),
                allocation("map texture", 10),
                allocation("map texture", 30),
                allocation("", 1),
            ],
            blocks: Vec::new(),
            total_allocated_bytes: 73 << 20,
            total_reserved_bytes: 96 << 20,
        };
        let lines = lines(Some(&report), Some("AMD Radeon, 495 MiB"));
        assert_eq!(lines[0], "vram: graphics card AMD Radeon, 495 MiB");
        assert_eq!(lines[1], "vram: 73 MiB in use, 96 MiB reserved in 0 blocks");
        assert!(lines[2].ends_with("2 x  map texture"), "{}", lines[2]);
        assert!(lines[2].contains("40.0 MiB"), "{}", lines[2]);
        assert!(lines[3].ends_with("1 x  fonts"), "{}", lines[3]);
        assert!(lines[4].ends_with("(unlabelled)"), "{}", lines[4]);
    }

    #[test]
    fn a_backend_without_a_report_says_so() {
        let lines = lines(None, None);
        assert_eq!(lines.len(), 2);
        assert!(lines[1].contains("no allocator report"));
    }

    #[test]
    fn labels_past_the_list_are_summed_in_one_line() {
        let report = wgpu::AllocatorReport {
            allocations: (0..30)
                .map(|i| allocation(&format!("t{i:02}"), 1))
                .collect(),
            blocks: Vec::new(),
            total_allocated_bytes: 30 << 20,
            total_reserved_bytes: 30 << 20,
        };
        let lines = lines(Some(&report), None);
        assert_eq!(lines.len(), 2 + LISTED + 1);
        assert!(lines.last().unwrap().contains("in 5 other labels"));
    }
}
