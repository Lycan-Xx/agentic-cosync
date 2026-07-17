use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

pub type PhysicalTime = u64;
pub type LogicalCounter = u64;
pub type DeviceId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct HlcTimestamp {
    pub physical_time_ms: PhysicalTime,
    pub logical_time: LogicalCounter,
    pub device_id: DeviceId,
}

impl HlcTimestamp {
    pub fn new(physical_time_ms: PhysicalTime, logical_time: LogicalCounter, device_id: DeviceId) -> Self {
        Self { physical_time_ms, logical_time, device_id }
    }
}

impl PartialOrd for HlcTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HlcTimestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.physical_time_ms.cmp(&other.physical_time_ms) {
            Ordering::Equal => match self.logical_time.cmp(&other.logical_time) {
                Ordering::Equal => self.device_id.cmp(&other.device_id),
                ord => ord,
            },
            ord => ord,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HybridLogicalClock {
    device_id: DeviceId,
    last: HlcTimestamp,
}

impl HybridLogicalClock {
    pub fn new(device_id: DeviceId) -> Self {
        let physical = Self::wall_clock_ms();
        let last = HlcTimestamp::new(physical, 0, device_id.clone());
        Self { device_id, last }
    }

    pub fn now(&mut self) -> HlcTimestamp {
        let wall = Self::wall_clock_ms();
        let last_pt = self.last.physical_time_ms;
        let last_lc = self.last.logical_time;
        let (pt, lc) = if wall > last_pt {
            (wall, 0)
        } else {
            (last_pt, last_lc + 1)
        };
        let ts = HlcTimestamp::new(pt, lc, self.device_id.clone());
        self.last = ts.clone();
        ts
    }

    pub fn receive(&mut self, msg: &HlcTimestamp) -> std::result::Result<HlcTimestamp, bool> {
        if msg.device_id == self.device_id {
            return Err(true);
        }
        let wall = Self::wall_clock_ms();
        let pt = wall.max(self.last.physical_time_ms).max(msg.physical_time_ms);
        let lc = if pt == msg.physical_time_ms {
            msg.logical_time + 1
        } else if pt == self.last.physical_time_ms {
            self.last.logical_time + 1
        } else {
            0
        };
        let ts = HlcTimestamp::new(pt, lc, self.device_id.clone());
        self.last = ts.clone();
        Ok(ts)
    }

    pub fn is_newer_than(&self, other: &HlcTimestamp) -> bool {
        other > &self.last
    }

    fn wall_clock_ms() -> PhysicalTime {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time before Unix epoch")
            .as_millis() as PhysicalTime
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_increments() {
        let mut hlc = HybridLogicalClock::new("device-a".into());
        let t1 = hlc.now();
        let t2 = hlc.now();
        assert!(t2 > t1 || t2.physical_time_ms > t1.physical_time_ms);
    }

    #[test]
    fn test_receive_merge() {
        let mut hlc_a = HybridLogicalClock::new("device-a".into());
        let mut hlc_b = HybridLogicalClock::new("device-b".into());
        let t_b = hlc_b.now();
        let received = hlc_a.receive(&t_b).unwrap();
        assert!(received >= t_b);
    }

    #[test]
    fn test_loop_detection() {
        let mut hlc = HybridLogicalClock::new("device-a".into());
        let t = hlc.now();
        let result = hlc.receive(&t);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), true);
    }

    #[test]
    fn test_ordering() {
        let t1 = HlcTimestamp::new(1000, 0, "a".into());
        let t2 = HlcTimestamp::new(1000, 1, "a".into());
        let t3 = HlcTimestamp::new(2000, 0, "a".into());
        assert!(t2 > t1);
        assert!(t3 > t2);
    }

    #[test]
    fn test_serde_roundtrip() {
        let ts = HlcTimestamp::new(1690000000000, 42, "device-x".into());
        let json = serde_json::to_string(&ts).unwrap();
        let deserialized: HlcTimestamp = serde_json::from_str(&json).unwrap();
        assert_eq!(ts, deserialized);
    }
}