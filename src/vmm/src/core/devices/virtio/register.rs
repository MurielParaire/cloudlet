use std::sync::{Arc, Mutex};

use super::{Error, MmioConfig, Result};
use linux_loader::cmdline;
use vm_device::{
    device_manager::{IoManager, MmioManager},
    DeviceMmio,
};
use vm_memory::{Address, GuestAddress, GuestUsize};

pub fn register_mmio_device(
    mmio_cfg: MmioConfig,
    device_mgr: Arc<Mutex<IoManager>>,
    irq: u32,
    id: Option<u32>,
    device: Arc<dyn DeviceMmio + Send + Sync>,
) -> Result<String> {
    device_mgr
        .lock()
        .map_err(|_| Error::Mutex)?
        .register_mmio(mmio_cfg.range, device)
        .map_err(Error::RegisterMmioDevice)?;

    let size = mmio_cfg.range.size();

    // Pass to kernel command line
    if size == 0 {
        return Err(Error::Cmdline(cmdline::Error::MmioSize));
    }

    let mut device_str = format!(
        "virtio_mmio.device={}@0x{:x?}:{}",
        guestusize_to_str(size),
        GuestAddress(mmio_cfg.range.base().0).raw_value(),
        irq
    );
    if let Some(id) = id {
        device_str.push_str(format!(":{}", id).as_str());
    }
    Ok(device_str)
}

fn guestusize_to_str(size: GuestUsize) -> String {
    const KB_MULT: u64 = 1 << 10;
    const MB_MULT: u64 = KB_MULT << 10;
    const GB_MULT: u64 = MB_MULT << 10;

    if size % GB_MULT == 0 {
        return format!("{}G", size / GB_MULT);
    }
    if size % MB_MULT == 0 {
        return format!("{}M", size / MB_MULT);
    }
    if size % KB_MULT == 0 {
        return format!("{}K", size / KB_MULT);
    }
    size.to_string()
}
