use sysinfo::System;

pub fn get_os_info() -> String {
    System::name().unwrap_or("UNknown OS".to_string())
}

pub fn get_sysinfo() {
    // Please note that we use "new_all" to ensure that all list of
    // components, network interfaces, disks and users are already
    // filled!
    let mut sys = System::new_all();

    // First we update all information of our `System` struct.
    sys.refresh_all();

    println!("=> system:");
    // RAM and swap information:
    println!("total memory: {} bytes", sys.total_memory());
    println!("used memory : {} bytes", sys.used_memory());
    println!("total swap  : {} bytes", sys.total_swap());
    println!("used swap   : {} bytes", sys.used_swap());

    // Display system information:
    println!("System name:             {:?}", System::name());
    println!("System kernel version:   {:?}", System::kernel_version());
    println!("System OS version:       {:?}", System::os_version());
    println!("System host name:        {:?}", System::host_name());

    // Number of CPUs:
    println!("NB CPUs: {}", sys.cpus().len());
}

#[cfg(test)]
mod tests {
    use super::get_os_info;

    #[test]
    fn test_get_os_info() {
        let os_info = get_os_info();
        println!("OS info: {}", os_info);
    }
}
