use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum XdpAction {
    Pass,
    Drop,
    Aborted,
    Tx,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XdpFilterRule {
    pub source_ip: String,
    pub max_packets_per_sec: u32,
    pub is_blocked: bool,
}

pub struct EbpfXdpFirewall {
    rules: HashMap<String, XdpFilterRule>,
    packet_counts: HashMap<String, u32>,
    dropped_count: u64,
}

impl Default for EbpfXdpFirewall {
    fn default() -> Self {
        Self::new()
    }
}

impl EbpfXdpFirewall {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            packet_counts: HashMap::new(),
            dropped_count: 0,
        }
    }

    pub fn add_rule(&mut self, source_ip: &str, max_pps: u32) {
        self.rules.insert(
            source_ip.to_string(),
            XdpFilterRule {
                source_ip: source_ip.to_string(),
                max_packets_per_sec: max_pps,
                is_blocked: false,
            },
        );
    }

    pub fn block_ip(&mut self, source_ip: &str) {
        if let Some(rule) = self.rules.get_mut(source_ip) {
            rule.is_blocked = true;
        } else {
            self.rules.insert(
                source_ip.to_string(),
                XdpFilterRule {
                    source_ip: source_ip.to_string(),
                    max_packets_per_sec: 0,
                    is_blocked: true,
                },
            );
        }
    }

    pub fn filter_packet(&mut self, source_ip: &str, _packet_size_bytes: usize) -> XdpAction {
        if let Some(rule) = self.rules.get(source_ip) {
            if rule.is_blocked {
                self.dropped_count += 1;
                return XdpAction::Drop;
            }

            let count = self.packet_counts.entry(source_ip.to_string()).or_insert(0);
            *count += 1;

            if rule.max_packets_per_sec > 0 && *count > rule.max_packets_per_sec {
                self.dropped_count += 1;
                return XdpAction::Drop;
            }
        }

        XdpAction::Pass
    }

    pub fn get_dropped_count(&self) -> u64 {
        self.dropped_count
    }

    pub fn reset_rate_limits(&mut self) {
        self.packet_counts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ebpf_xdp_firewall_rate_limiting() {
        let mut firewall = EbpfXdpFirewall::new();
        firewall.add_rule("192.168.1.50", 2);

        assert_eq!(firewall.filter_packet("192.168.1.50", 64), XdpAction::Pass);
        assert_eq!(firewall.filter_packet("192.168.1.50", 64), XdpAction::Pass);
        assert_eq!(firewall.filter_packet("192.168.1.50", 64), XdpAction::Drop);
        assert_eq!(firewall.get_dropped_count(), 1);
    }

    #[test]
    fn test_ebpf_xdp_firewall_ip_blocking() {
        let mut firewall = EbpfXdpFirewall::new();
        firewall.block_ip("10.0.0.99");

        assert_eq!(firewall.filter_packet("10.0.0.99", 128), XdpAction::Drop);
        assert_eq!(firewall.get_dropped_count(), 1);
    }
}
