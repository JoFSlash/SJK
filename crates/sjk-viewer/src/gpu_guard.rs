//! GPU work that may run out of video memory without ending the game.
//!
//! wgpu reports an error that no error scope captures to the device's default
//! handler, which panics ("wgpu error: Out of Memory"). Work done after a world is
//! built, on the render thread (the menu stage's model, a player's textures), runs
//! through [`capture`] instead, so a card short of memory loses that one model and
//! the game goes on. What the failed work made is invalid and must be dropped, not
//! kept in a cache: a later draw that binds it fails validation.

/// Run `create` with wgpu's out-of-memory and validation errors captured: the
/// first one wgpu reported instead of the value.
pub(crate) fn capture<T>(
    device: &wgpu::Device,
    create: impl FnOnce() -> T,
) -> Result<T, wgpu::Error> {
    let memory = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let value = create();
    let invalid = pollster::block_on(validation.pop());
    let exhausted = pollster::block_on(memory.pop());
    match invalid.or(exhausted) {
        Some(error) => Err(error),
        None => Ok(value),
    }
}

/// Make the errors nothing captures panic with what failed, not wgpu's bare "Out of
/// Memory": the operation and the label ("In Device::create_texture, label = '...'") on
/// the first line, which the crash report's signature reads, then the causes and how
/// much video memory SJK held. Small cards' crashes said only "Out of Memory" (Vin's and
/// Kushina's, 21/10/2026).
pub(crate) fn name_uncaptured(device: &wgpu::Device) {
    let held = device.clone();
    device.on_uncaptured_error(std::sync::Arc::new(move |error: wgpu::Error| {
        panic!("{}", described(&error, held.generate_allocator_report()));
    }));
}

/// The panic message for `error`, with the allocator's `report` when there is one.
fn described(error: &wgpu::Error, report: Option<wgpu::AllocatorReport>) -> String {
    use std::error::Error as _;
    let kind = match error {
        wgpu::Error::OutOfMemory { .. } => "Out of Memory",
        wgpu::Error::Validation { .. } => "Validation Error",
        wgpu::Error::Internal { .. } => "Internal Error",
    };
    let mut causes = Vec::new();
    let mut cause = error.source();
    while let Some(next) = cause {
        causes.push(next.to_string());
        cause = next.source();
    }
    let mut text = match causes.first() {
        Some(context) => format!("wgpu error: {kind} ({context})"),
        None => format!("wgpu error: {kind}"),
    };
    for cause in causes.iter().skip(1) {
        text.push_str(&format!("\n  {cause}"));
    }
    if let wgpu::Error::Validation { description, .. } | wgpu::Error::Internal { description, .. } =
        error
    {
        text.push_str(&format!("\n\n{}", description.trim_end()));
    }
    if let Some(report) = report {
        const MIB: u64 = 1 << 20;
        text.push_str(&format!(
            "\n\nvideo memory held by SJK: {} MiB in use, {} MiB reserved in {} blocks",
            report.total_allocated_bytes / MIB,
            report.total_reserved_bytes / MIB,
            report.blocks.len()
        ));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Cause(&'static str, Option<Box<Cause>>);

    impl std::fmt::Display for Cause {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for Cause {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            self.1.as_deref().map(|cause| cause as _)
        }
    }

    #[test]
    fn an_uncaptured_error_names_what_failed() {
        let error = wgpu::Error::OutOfMemory {
            source: Box::new(Cause(
                "In Device::create_texture, label = 'menu picture'",
                Some(Box::new(Cause("Not enough memory left.", None))),
            )),
        };
        assert_eq!(
            described(&error, None),
            "wgpu error: Out of Memory (In Device::create_texture, label = 'menu picture')\n  \
             Not enough memory left."
        );
    }
}
