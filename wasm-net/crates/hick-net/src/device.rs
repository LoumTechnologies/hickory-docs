//! Virtual network device backed by in-memory queues.
//!
//! Implements smoltcp's [`Device`] trait using `VecDeque<Vec<u8>>` for both
//! receive and transmit directions. The host side pushes frames via
//! [`VirtualDevice::inject_frame`] and collects outbound frames via
//! [`VirtualDevice::drain_tx`].

use std::collections::VecDeque;

use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::time::Instant;

use crate::net_config::MTU;

/// A virtual Ethernet device backed by in-memory frame queues.
pub struct VirtualDevice {
    rx_queue: VecDeque<Vec<u8>>,
    tx_queue: VecDeque<Vec<u8>>,
}

impl Default for VirtualDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualDevice {
    pub fn new() -> Self {
        Self {
            rx_queue: VecDeque::new(),
            tx_queue: VecDeque::new(),
        }
    }

    /// Push a raw Ethernet frame into the receive queue (from the wire side).
    pub fn inject_frame(&mut self, frame: Vec<u8>) {
        self.rx_queue.push_back(frame);
    }

    /// Pop the next outbound frame from the transmit queue.
    pub fn drain_tx(&mut self) -> Option<Vec<u8>> {
        self.tx_queue.pop_front()
    }

    /// Drain all outbound frames from the transmit queue.
    pub fn drain_all_tx(&mut self) -> Vec<Vec<u8>> {
        self.tx_queue.drain(..).collect()
    }

    /// Number of frames waiting in the receive queue.
    pub fn rx_pending(&self) -> usize {
        self.rx_queue.len()
    }

    /// Number of frames waiting in the transmit queue.
    pub fn tx_pending(&self) -> usize {
        self.tx_queue.len()
    }
}

impl Device for VirtualDevice {
    type RxToken<'a> = VirtualRxToken;
    type TxToken<'a> = VirtualTxToken<'a>;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let frame = self.rx_queue.pop_front()?;
        Some((
            VirtualRxToken { buffer: frame },
            VirtualTxToken {
                queue: &mut self.tx_queue,
            },
        ))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(VirtualTxToken {
            queue: &mut self.tx_queue,
        })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ethernet;
        caps.max_transmission_unit = MTU;
        caps.max_burst_size = Some(1);
        caps
    }
}

/// Receive token that owns a single inbound frame.
pub struct VirtualRxToken {
    buffer: Vec<u8>,
}

impl RxToken for VirtualRxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.buffer)
    }
}

/// Transmit token that borrows the device's TX queue.
pub struct VirtualTxToken<'a> {
    queue: &'a mut VecDeque<Vec<u8>>,
}

impl TxToken for VirtualTxToken<'_> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buffer = vec![0u8; len];
        let result = f(&mut buffer);
        self.queue.push_back(buffer);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_inject_and_receive() {
        let mut dev = VirtualDevice::new();
        let frame = vec![0xAA, 0xBB, 0xCC, 0xDD];
        dev.inject_frame(frame.clone());

        assert_eq!(dev.rx_pending(), 1);

        let (rx, _tx) = dev.receive(Instant::ZERO).expect("should have a frame");
        let received = rx.consume(|buf| buf.to_vec());
        assert_eq!(received, frame);
        assert_eq!(dev.rx_pending(), 0);
    }

    #[test]
    fn test_device_transmit() {
        let mut dev = VirtualDevice::new();
        let tx = dev.transmit(Instant::ZERO).expect("should always succeed");

        let payload = b"outbound frame";
        tx.consume(payload.len(), |buf| {
            buf.copy_from_slice(payload);
        });

        assert_eq!(dev.tx_pending(), 1);
        let out = dev.drain_tx().unwrap();
        assert_eq!(out, payload);
    }

    #[test]
    fn test_device_empty_receive_returns_none() {
        let mut dev = VirtualDevice::new();
        assert!(dev.receive(Instant::ZERO).is_none());
    }

    #[test]
    fn test_device_capabilities() {
        let dev = VirtualDevice::new();
        let caps = dev.capabilities();
        assert_eq!(caps.max_transmission_unit, MTU);
        assert_eq!(caps.medium, Medium::Ethernet);
    }
}
