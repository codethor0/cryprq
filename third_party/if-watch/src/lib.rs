// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! Minimal tokio-based IP address watcher for CrypRQ
#![deny(missing_docs)]
#![deny(warnings)]

use if_addrs::IfAddr;
pub use ipnet::{IpNet, Ipv4Net, Ipv6Net};

/// An address change event.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IfEvent {
    /// A new local address has been added.
    Up(IpNet),
    /// A local address has been deleted.
    Down(IpNet),
}

fn ifaddr_to_ipnet(addr: IfAddr) -> IpNet {
    match addr {
        IfAddr::V4(ip) => {
            let prefix_len = (!u32::from_be_bytes(ip.netmask.octets())).leading_zeros();
            IpNet::V4(
                Ipv4Net::new(ip.ip, prefix_len as u8).expect("if_addrs returned a valid prefix"),
            )
        }
        IfAddr::V6(ip) => {
            let prefix_len = (!u128::from_be_bytes(ip.netmask.octets())).leading_zeros();
            IpNet::V6(
                Ipv6Net::new(ip.ip, prefix_len as u8).expect("if_addrs returned a valid prefix"),
            )
        }
    }
}

/// Tokio-enabled network interface watcher.
pub mod tokio {
    // © 2025 Thor Thor
    // Contact: codethor@gmail.com
    // LinkedIn: https://www.linkedin.com/in/thor-thor0
    // SPDX-License-Identifier: MIT

    use super::{ifaddr_to_ipnet, IfEvent};
    use futures::{ready, stream::FusedStream, Stream};
    use ipnet::IpNet;
    use std::collections::{HashSet, VecDeque};
    use std::io::Result;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use std::time::Duration;
    use tokio::time::{self, Instant, Interval};

    /// Watches for interface changes by polling the system address list.
    #[derive(Debug)]
    pub struct IfWatcher {
        addrs: HashSet<IpNet>,
        queue: VecDeque<IfEvent>,
        ticker: Interval,
    }

    impl IfWatcher {
        /// Create a watcher that polls every five seconds.
        pub fn new() -> Result<Self> {
            let ticker = time::interval_at(Instant::now(), Duration::from_secs(5));
            let mut watcher = Self {
                addrs: HashSet::new(),
                queue: VecDeque::new(),
                ticker,
            };
            watcher.resync()?;
            Ok(watcher)
        }

        fn resync(&mut self) -> Result<()> {
            let addrs = if_addrs::get_if_addrs()?;
            let next: HashSet<IpNet> = addrs.into_iter().map(|addr| ifaddr_to_ipnet(addr.addr)).collect();

            for old_addr in self.addrs.iter().cloned() {
                if !next.contains(&old_addr) {
                    self.queue.push_back(IfEvent::Down(old_addr));
                }
            }

            for new_addr in next.iter().cloned() {
                if !self.addrs.contains(&new_addr) {
                    self.queue.push_back(IfEvent::Up(new_addr));
                }
            }

            self.addrs = next;
            Ok(())
        }

        /// Iterate over the currently known addresses.
        pub fn iter(&self) -> impl Iterator<Item = &IpNet> {
            self.addrs.iter()
        }

        /// Poll the watcher for the next event (exposed for libp2p compatibility).
        pub fn poll_if_event(&mut self, cx: &mut Context<'_>) -> Poll<Result<IfEvent>> {
            loop {
                if let Some(event) = self.queue.pop_front() {
                    return Poll::Ready(Ok(event));
                }
                ready!(Pin::new(&mut self.ticker).poll_tick(cx));
                self.resync()?;
            }
        }
    }

    impl Stream for IfWatcher {
        type Item = Result<IfEvent>;

        fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            match self.get_mut().poll_if_event(cx) {
                Poll::Ready(Ok(event)) => Poll::Ready(Some(Ok(event))),
                Poll::Ready(Err(err)) => Poll::Ready(Some(Err(err))),
                Poll::Pending => Poll::Pending,
            }
        }
    }

    impl FusedStream for IfWatcher {
        fn is_terminated(&self) -> bool {
            false
        }
    }
}

/// `smol` runtime is not supported in this vendored build.
#[cfg(feature = "smol")]
pub mod smol {
    compile_error!("smol runtime is not supported in the CrypRQ vendored if-watch crate");
}
