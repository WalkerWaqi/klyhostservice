use log::info;
use virt::connect::Connect;
use virt::error::Error;

#[allow(dead_code)]
pub struct Virt {
    conn: Connect,
}

#[allow(dead_code)]
impl Virt {
    pub fn new(uri: &str) -> Self {
        Virt {
            conn: Connect::open(uri).unwrap(),
        }
    }

    pub fn show_hypervisor_info(&self) -> Result<(), Error> {
        if let Ok(hv_type) = self.conn.get_type() {
            if let Ok(mut hv_ver) = self.conn.get_hyp_version() {
                let major = hv_ver / 1000000;
                hv_ver %= 1000000;
                let minor = hv_ver / 1000;
                let release = hv_ver % 1000;
                info!(
                    "Hypervisor: '{}' version: {}.{}.{}",
                    hv_type, major, minor, release
                );
                return Ok(());
            }
        }
        Err(Error::new())
    }

    pub fn show_domains(&self) -> Result<(), Error> {
        let flags = virt::connect::VIR_CONNECT_LIST_DOMAINS_ACTIVE
            | virt::connect::VIR_CONNECT_LIST_DOMAINS_INACTIVE;
        if let Ok(num_active_domains) = self.conn.num_of_domains() {
            if let Ok(num_inactive_domains) = self.conn.num_of_defined_domains() {
                info!(
                    "There are {} active and {} inactive domains",
                    num_active_domains, num_inactive_domains
                );
                /* Return a list of all active and inactive domains. Using this API
                 * instead of virConnectListDomains() and virConnectListDefinedDomains()
                 * is preferred since it "solves" an inherit race between separated API
                 * calls if domains are started or stopped between calls */
                if let Ok(doms) = self.conn.list_all_domains(flags) {
                    for dom in doms {
                        let id = dom.get_id().unwrap_or(0);
                        let name = dom.get_name().unwrap_or(String::from("no-name"));
                        let active = dom.is_active().unwrap_or(false);
                        info!("ID: {}, Name: {}, Active: {}", id, name, active);
                        if let Ok(dinfo) = dom.get_info() {
                            info!("Domain info:");
                            info!("    State: {}", dinfo.state);
                            info!("    Max Memory: {}", dinfo.max_mem);
                            info!("    Memory: {}", dinfo.memory);
                            info!("    CPUs: {}", dinfo.nr_virt_cpu);
                            info!("    CPU Time: {}", dinfo.cpu_time);
                        }
                        if let Ok(memtune) = dom.get_memory_parameters(0) {
                            info!("Memory tune:");
                            info!("    Hard Limit: {}", memtune.hard_limit.unwrap_or(0));
                            info!("    Soft Limit: {}", memtune.soft_limit.unwrap_or(0));
                            info!("    Min Guarantee: {}", memtune.min_guarantee.unwrap_or(0));
                            info!(
                                "    Swap Hard Limit: {}",
                                memtune.swap_hard_limit.unwrap_or(0)
                            );
                        }
                        if let Ok(numa) = dom.get_numa_parameters(0) {
                            info!("NUMA:");
                            info!(
                                "    Node Set: {}",
                                numa.node_set.unwrap_or(String::from(""))
                            );
                            info!("    Mode: {}", numa.mode.unwrap_or(0));
                        }
                    }
                }
                return Ok(());
            }
        }
        Err(Error::new())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn nothing() {
        let virt = super::Virt::new("qemu:///system");
        let _ = virt.show_hypervisor_info();
        let _ = virt.show_domains();
    }
}
