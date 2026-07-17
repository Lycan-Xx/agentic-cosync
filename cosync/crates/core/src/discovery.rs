use crate::error::{CosyncError, Result};
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashMap;
use std::net::{IpAddr, UdpSocket};
use std::sync::Arc;
use tokio::sync::mpsc;

pub const SERVICE_TYPE: &str = "_cosync._udp.local";

#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    pub device_name: String,
    pub fingerprint: String,
    pub addresses: Vec<IpAddr>,
    pub port: u16,
}

pub struct DiscoveryService {
    daemon: Arc<ServiceDaemon>,
}

impl DiscoveryService {
    pub fn new() -> Result<Self> {
        let daemon = ServiceDaemon::new()
            .map_err(|e| CosyncError::Discovery(format!("Failed to create mDNS daemon: {}", e)))?;
        Ok(Self { daemon: Arc::new(daemon) })
    }

    pub fn advertise(&self, instance_name: &str, port: u16, fingerprint: &str) -> Result<ServiceInfo> {
        let mut properties = HashMap::new();
        properties.insert("fp".to_string(), fingerprint.to_string());

        let service_info = ServiceInfo::new(
            SERVICE_TYPE, instance_name,
            &format!("{}.local.", instance_name.to_lowercase().replace(' ', "-")),
            "", port, properties,
        ).map_err(|e| CosyncError::Discovery(format!("Failed to create ServiceInfo: {}", e)))?
         .enable_addr_auto();

        self.daemon.register(service_info.clone())
            .map_err(|e| CosyncError::Discovery(format!("Failed to register: {}", e)))?;
        tracing::info!(device = instance_name, "Advertising on mDNS");
        Ok(service_info)
    }

    pub fn stop_advertising(&self, fullname: &str) -> Result<()> {
        self.daemon.unregister(fullname)
            .map_err(|e| CosyncError::Discovery(format!("Failed to unregister: {}", e)))?;
        Ok(())
    }

    pub fn browse(&self) -> Result<mpsc::Receiver<DiscoveredPeer>> {
        let (tx, rx) = mpsc::channel(32);
        let receiver = self.daemon.browse(SERVICE_TYPE)
            .map_err(|e| CosyncError::Discovery(format!("Failed to browse: {}", e)))?;

        std::thread::spawn(move || {
            loop {
                match receiver.recv() {
                    Ok(ServiceEvent::ServiceResolved(info)) => {
                        let peer = DiscoveredPeer {
                            device_name: info.get_fullname().to_string(),
                            fingerprint: info.get_property_val_str("fp").unwrap_or("").to_string(),
                            addresses: info.get_addresses().iter().cloned().collect(),
                            port: info.get_port(),
                        };
                        if tx.blocking_send(peer).is_err() { break; }
                    }
                    Ok(ServiceEvent::ServiceRemoved(_, name)) => {
                        tracing::debug!(peer = %name, "Peer removed from mDNS");
                    }
                    Err(e) => { tracing::warn!("mDNS browse error: {}", e); break; }
                    _ => {}
                }
            }
        });
        Ok(rx)
    }

    pub fn shutdown(&self) -> Result<()> {
        self.daemon.shutdown()
            .map_err(|e| CosyncError::Discovery(format!("Failed to shutdown: {}", e)))?;
        Ok(())
    }
}

pub fn get_local_ip() -> Result<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| CosyncError::Discovery(format!("UDP bind failed: {}", e)))?;
    socket.connect("8.8.8.8:80")
        .map_err(|e| CosyncError::Discovery(format!("UDP connect failed: {}", e)))?;
    let local_addr = socket.local_addr()
        .map_err(|e| CosyncError::Discovery(format!("local_addr failed: {}", e)))?;
    Ok(local_addr.ip())
}

impl Default for DiscoveryService {
    fn default() -> Self { Self::new().expect("Failed to create DiscoveryService") }
}