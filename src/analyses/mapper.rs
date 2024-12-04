use crate::representations::arch::*;
use crate::representations::loop_prob::*;

struct Mapper {
    arch: Arch,
    loop_prob: LoopProb,
}

impl Mapper {
    pub fn new(arch: Arch, loop_prob: LoopProb) -> Self {
        Self { arch, loop_prob }
    }

    /// Map all data accesses in the loop nest to the architecture's data ports.
    /// Priotizing the use of the NOC data ports.
    /// So far only handling the case where the total number of data ports is more than the number of data accesses.
    pub fn map_max_noc(&self) {
        let mut data_ports = self.arch.pe_arch.data_ports.iter();
        let mut data_accesses = self.loop_prob.body.iter();
        let mut data_port = data_ports.next();
        let mut data_access = data_accesses.next();
        while data_port.is_some() && data_access.is_some() {
            match data_port.unwrap() {
                DataPort::NOC(noc) => {
                    println!("Mapping {:?} to NOC {}", data_access.unwrap(), noc.name);
                    data_access = data_accesses.next();
                }
                DataPort::Memory(memory) => {
                    println!(
                        "Mapping {:?} to Memory {}",
                        data_access.unwrap(),
                        memory.name
                    );
                    data_access = data_accesses.next();
                }
            }
            data_port = data_ports.next();
        }
    }
}
