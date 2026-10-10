//! Isolated protocol-only probe. An empty Wayland surface is NOT a rendering proof.
#[path = "../owned_wayland_presentation.rs"]
mod owned_wayland_presentation;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut owned = owned_wayland_presentation::OwnedWaylandPresentation::open()?;
    println!("GPU05_OWNED_SURFACE_CONFIGURED clock={:?} pending={} (NO_GPU_PRESENTATION)",
        owned.clock_id(), owned.pending_count());
    let feedback = owned.diagnostic_roundtrip()?;
    println!("GPU05_DIAGNOSTIC_FEEDBACK={} (NO_GPU_PRESENTATION)", feedback.len());
    Ok(())
}
