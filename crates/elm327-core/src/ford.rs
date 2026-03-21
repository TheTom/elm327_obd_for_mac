//! Ford-specific module database — mapping module names to CAN addresses.
//!
//! Currently targets the 2017 F-150 (2015-2020 platform). CAN IDs are
//! approximate and based on common Ford UDS addresses. They'll be verified
//! against the actual truck.
//!
//! # Usage
//! ```
//! use elm327_core::ford::{find_module, modules_for_bus, CanBus};
//!
//! let pcm = find_module("PCM").unwrap();
//! assert_eq!(pcm.request_id, 0x7E0);
//!
//! let ms_can = modules_for_bus(CanBus::MsCan);
//! assert!(ms_can.len() > 0);
//! ```

/// A Ford vehicle module with its CAN bus addressing info.
#[derive(Debug, Clone)]
pub struct FordModule {
    pub name: &'static str,
    pub abbreviation: &'static str,
    /// CAN ID to send requests to
    pub request_id: u16,
    /// CAN ID responses come from (typically request_id + 8)
    pub response_id: u16,
    pub bus: CanBus,
    pub description: &'static str,
}

/// Which CAN bus a module lives on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CanBus {
    /// High Speed CAN (500 kbps, OBD-II pins 6/14)
    HsCan,
    /// Medium Speed CAN (125 kbps, OBD-II pins 3/11)
    MsCan,
}

/// Known Ford module addresses for F-150 (2015-2020).
///
/// Standard UDS addressing: request = 0x7XX, response = request + 0x08.
///
/// TODO: Verify all CAN IDs against the actual 2017 F-150 with FORScan
/// TODO: Add extended addressing modules (0x18DAXXFF style) if needed
pub static FORD_MODULES: &[FordModule] = &[
    // ── HS-CAN powertrain ───────────────────────────────────────────────
    FordModule {
        name: "Powertrain Control Module",
        abbreviation: "PCM",
        request_id: 0x7E0,
        response_id: 0x7E8,
        bus: CanBus::HsCan,
        description: "Engine and transmission control",
    },
    FordModule {
        name: "Transmission Control Module",
        abbreviation: "TCM",
        request_id: 0x7E1,
        response_id: 0x7E9,
        bus: CanBus::HsCan,
        description: "Automatic transmission",
    },
    // ── HS-CAN chassis ──────────────────────────────────────────────────
    FordModule {
        name: "Anti-lock Brake System",
        abbreviation: "ABS",
        request_id: 0x760,
        response_id: 0x768,
        bus: CanBus::HsCan,
        description: "ABS and stability control",
    },
    FordModule {
        name: "Restraint Control Module",
        abbreviation: "RCM",
        request_id: 0x737,
        response_id: 0x73F,
        bus: CanBus::HsCan,
        description: "Airbags and seatbelt pretensioners",
    },
    FordModule {
        name: "Parking Aid Module",
        abbreviation: "PAM",
        request_id: 0x736,
        response_id: 0x73E,
        bus: CanBus::HsCan,
        description: "Park assist sensors",
    },
    FordModule {
        name: "Power Steering Control Module",
        abbreviation: "PSCM",
        request_id: 0x730,
        response_id: 0x738,
        bus: CanBus::HsCan,
        description: "Electric power steering",
    },
    FordModule {
        name: "Adaptive Cruise Control",
        abbreviation: "ACC",
        request_id: 0x764,
        response_id: 0x76C,
        bus: CanBus::HsCan,
        description: "Radar cruise control",
    },
    FordModule {
        name: "Image Processing Module A",
        abbreviation: "IPMA",
        request_id: 0x706,
        response_id: 0x70E,
        bus: CanBus::HsCan,
        description: "Camera system",
    },
    FordModule {
        name: "Headlamp Control Module",
        abbreviation: "HCM",
        request_id: 0x753,
        response_id: 0x75B,
        bus: CanBus::HsCan,
        description: "Headlamp leveling/adaptive",
    },
    // ── HS-CAN body ─────────────────────────────────────────────────────
    FordModule {
        name: "Body Control Module",
        abbreviation: "BCM",
        request_id: 0x726,
        response_id: 0x72E,
        bus: CanBus::HsCan,
        description: "Lighting, locks, windows, wipers",
    },
    FordModule {
        name: "Instrument Panel Cluster",
        abbreviation: "IPC",
        request_id: 0x720,
        response_id: 0x728,
        bus: CanBus::HsCan,
        description: "Dashboard gauges and display",
    },
    FordModule {
        name: "APIM (Sync)",
        abbreviation: "APIM",
        request_id: 0x7D0,
        response_id: 0x7D8,
        bus: CanBus::HsCan,
        description: "Sync infotainment system",
    },
    FordModule {
        name: "Audio Control Module",
        abbreviation: "ACM",
        request_id: 0x754,
        response_id: 0x75C,
        bus: CanBus::HsCan,
        description: "Audio amplifier",
    },
    FordModule {
        name: "Trailer Brake Control Module",
        abbreviation: "TBC",
        request_id: 0x762,
        response_id: 0x76A,
        bus: CanBus::HsCan,
        description: "Integrated trailer brake controller",
    },
    FordModule {
        name: "Gateway Module",
        abbreviation: "GWM",
        request_id: 0x716,
        response_id: 0x71E,
        bus: CanBus::HsCan,
        description: "CAN bus gateway",
    },
    FordModule {
        name: "All Terrain Control Module",
        abbreviation: "ATCM",
        request_id: 0x765,
        response_id: 0x76D,
        bus: CanBus::HsCan,
        description: "4WD / AWD control",
    },
    FordModule {
        name: "Occupant Classification System",
        abbreviation: "OCS",
        request_id: 0x793,
        response_id: 0x79B,
        bus: CanBus::HsCan,
        description: "Passenger seat sensor",
    },
    // ── MS-CAN modules (need toggle switch + Protocol B) ────────────────
    FordModule {
        name: "Driver Door Module",
        abbreviation: "DDM",
        request_id: 0x740,
        response_id: 0x748,
        bus: CanBus::HsCan,
        description: "Driver door controls",
    },
    FordModule {
        name: "Passenger Door Module",
        abbreviation: "PDM",
        request_id: 0x742,
        response_id: 0x74A,
        bus: CanBus::HsCan,
        description: "Passenger door controls",
    },
    FordModule {
        name: "Rear View Camera Module",
        abbreviation: "RVCM",
        request_id: 0x752,
        response_id: 0x75A,
        bus: CanBus::MsCan,
        description: "Backup camera",
    },
    FordModule {
        name: "HVAC Control Module",
        abbreviation: "FHCM",
        request_id: 0x733,
        response_id: 0x73B,
        bus: CanBus::MsCan,
        description: "Climate control",
    },
    FordModule {
        name: "Seat Control Module Driver",
        abbreviation: "DSCM",
        request_id: 0x744,
        response_id: 0x74C,
        bus: CanBus::MsCan,
        description: "Power seat motors",
    },
];

/// Look up a module by its abbreviation (case-sensitive).
pub fn find_module(abbreviation: &str) -> Option<&'static FordModule> {
    FORD_MODULES.iter().find(|m| m.abbreviation == abbreviation)
}

/// Get all modules that live on a specific CAN bus.
pub fn modules_for_bus(bus: CanBus) -> Vec<&'static FordModule> {
    FORD_MODULES.iter().filter(|m| m.bus == bus).collect()
}

/// Get all HS-CAN modules.
pub fn hs_can_modules() -> Vec<&'static FordModule> {
    modules_for_bus(CanBus::HsCan)
}

/// Get all MS-CAN modules (require OBD-II switch to pins 3/11).
pub fn ms_can_modules() -> Vec<&'static FordModule> {
    modules_for_bus(CanBus::MsCan)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_module() {
        let pcm = find_module("PCM").expect("PCM should exist");
        assert_eq!(pcm.request_id, 0x7E0);
        assert_eq!(pcm.response_id, 0x7E8);
        assert_eq!(pcm.bus, CanBus::HsCan);

        let bcm = find_module("BCM").expect("BCM should exist");
        assert_eq!(bcm.request_id, 0x726);
        assert_eq!(bcm.response_id, 0x72E);

        // Non-existent module returns None
        assert!(find_module("NOPE").is_none());
    }

    #[test]
    fn test_modules_for_bus() {
        let hs = modules_for_bus(CanBus::HsCan);
        let ms = modules_for_bus(CanBus::MsCan);

        // Every HS module should actually be on HS-CAN
        for m in &hs {
            assert_eq!(m.bus, CanBus::HsCan, "{} should be HS-CAN", m.abbreviation);
        }
        // Every MS module should actually be on MS-CAN
        for m in &ms {
            assert_eq!(m.bus, CanBus::MsCan, "{} should be MS-CAN", m.abbreviation);
        }

        // Together they should account for all modules
        assert_eq!(hs.len() + ms.len(), FORD_MODULES.len());

        // Sanity check known counts
        assert!(hs.len() > ms.len(), "HS-CAN should have more modules than MS-CAN");
        assert_eq!(ms.len(), 3, "Expected 3 MS-CAN modules");
    }

    #[test]
    fn test_module_ids() {
        // Standard UDS: response_id = request_id + 8
        for m in FORD_MODULES.iter() {
            assert_eq!(
                m.response_id,
                m.request_id + 8,
                "Module {} ({}) response_id should be request_id + 8",
                m.name,
                m.abbreviation,
            );
        }
    }

    #[test]
    fn test_all_modules_have_names() {
        for m in FORD_MODULES.iter() {
            assert!(!m.name.is_empty(), "Module name must not be empty");
            assert!(!m.abbreviation.is_empty(), "Abbreviation must not be empty");
            assert!(!m.description.is_empty(), "Description must not be empty");
        }
    }

    // ── 2017 F-150 specific module tests ──────────────────────────────────

    #[test]
    fn test_pcm_addresses() {
        let pcm = find_module("PCM").unwrap();
        assert_eq!(pcm.request_id, 0x7E0);
        assert_eq!(pcm.response_id, 0x7E8);
        assert_eq!(pcm.bus, CanBus::HsCan);
    }

    #[test]
    fn test_bcm_exists() {
        let bcm = find_module("BCM").unwrap();
        assert_eq!(bcm.bus, CanBus::HsCan);
    }

    #[test]
    fn test_tcm_exists() {
        let tcm = find_module("TCM").unwrap();
        assert_eq!(tcm.request_id, 0x7E1);
        assert_eq!(tcm.response_id, 0x7E9);
        assert_eq!(tcm.bus, CanBus::HsCan);
    }

    #[test]
    fn test_all_response_ids_offset_by_8() {
        for module in FORD_MODULES {
            assert_eq!(
                module.response_id,
                module.request_id + 8,
                "Module {} has wrong response ID offset",
                module.abbreviation
            );
        }
    }

    #[test]
    fn test_f150_has_at_least_20_modules() {
        // The FORScan profile shows 22 modules for the 2017 F-150.
        // We may not have exactly 22 yet, but should have at least 20.
        assert!(
            FORD_MODULES.len() >= 20,
            "Expected at least 20 modules, got {}",
            FORD_MODULES.len()
        );
    }

    #[test]
    fn test_hs_can_modules_majority() {
        let hs = hs_can_modules();
        let ms = ms_can_modules();
        assert!(
            hs.len() > ms.len(),
            "HS-CAN should have more modules than MS-CAN (HS={}, MS={})",
            hs.len(),
            ms.len()
        );
    }

    #[test]
    fn test_no_duplicate_abbreviations() {
        let mut seen = std::collections::HashSet::new();
        for m in FORD_MODULES {
            assert!(
                seen.insert(m.abbreviation),
                "Duplicate module abbreviation: {}",
                m.abbreviation
            );
        }
    }

    #[test]
    fn test_no_duplicate_request_ids() {
        let mut seen = std::collections::HashSet::new();
        for m in FORD_MODULES {
            assert!(
                seen.insert(m.request_id),
                "Duplicate request_id 0x{:03X} for module {}",
                m.request_id,
                m.abbreviation
            );
        }
    }

    #[test]
    fn test_critical_modules_exist() {
        // Every 2017 F-150 should have these modules
        let required = vec!["PCM", "TCM", "ABS", "BCM", "IPC", "RCM"];
        for abbrev in required {
            assert!(
                find_module(abbrev).is_some(),
                "Critical module {} should exist in database",
                abbrev
            );
        }
    }
}
