use procfs;
use procfs::process::FDTarget;
use procfs::process::Stat;
use std::collections::HashMap;


pub fn get_processes() -> HashMap<u64, Stat> {
    /* gets all running processes on the system */

    let all_procs = procfs::process::all_processes().unwrap();

    let mut map: HashMap<u64, Stat> = HashMap::new();
    for p in all_procs {
        let process = p.unwrap();
        if let (Ok(stat), Ok(fds)) = (process.stat(), process.fd()) {
            for fd in fds {
                if let FDTarget::Socket(inode) = fd.unwrap().target {
                    map.insert(inode, stat.clone());
                }
            }
        }
    }
    return map;
}
