use crate::representations::arch::*;
use crate::representations::loops::*;

struct Mapper {
    arch: Arch,
    loop_prob: LoopNest,
}

impl Mapper {
    pub fn new(arch: Arch, loop_prob: LoopNest) -> Self {
        Self { arch, loop_prob }
    }

    /// Helper: Check if two topologies are symmetric opposites
    fn is_symmetric_opposites(topo1: &[i32], topo2: &[i32]) -> bool {
        if topo1.len() != topo2.len() {
            return false;
        }
        topo1.iter().zip(topo2.iter()).all(|(a, b)| a + b == 0)
    }

    fn get_noc_ports(&self) -> Vec<&NocPort> {
        self.arch
            .pe_arch
            .data_ports
            .iter()
            .filter_map(|port| match port {
                DataPort::NocPort(noc) => Some(noc),
                _ => None,
            })
            .collect()
    }

    /// For each NOC port, there needs be a corresponding NOC in the opposite direction.
    /// E.g. for [0,1] (NORTH), there needs to be a [0,-1] (SOUTH).
    fn check_noc_symetry(&self) -> bool {
        // Get all NOC ports from all data ports
        let noc_ports = self.get_noc_ports();

        // For each port, check if it has a symmetric opposite
        noc_ports.iter().all(|port1| {
            noc_ports.iter().any(|port2| {
                if std::ptr::eq(*port1, *port2) {
                    return false; // Skip comparing port with itself
                }
                Mapper::is_symmetric_opposites(&port1.topology, &port2.topology)
            })
        })
    }

    /// Find the NOC port that is symmetric opposite to the given NOC port
    fn find_symmetric_noc(&self, noc: &NocPort) -> Option<&NocPort> {
        let noc_ports = self.get_noc_ports();
        noc_ports
            .iter()
            .find(|&noc2| Mapper::is_symmetric_opposites(&noc.topology, &noc2.topology))
            .map(|v| &**v)
    }

    /// Compute the "rank" of the NOC ports, i.e. the number of dimensions of the "flow"
    /// For instance, a 2D mesh has a rank of 2.
    /// If a 2D mesh has 2 ports in each direction (total of 8 ports), its rank is 4
    fn noc_rank(&self) -> usize {
        let noc_ports = self.get_noc_ports();
        assert!(
            self.check_noc_symetry(),
            "You can only get the rank if the NOC is symmetrical"
        );
        noc_ports.len() / 2
    }

    /// For all data accesses in the loop body, generate all possible mappings to the data ports
    pub fn generate_all_mappings(&self) {}
}
