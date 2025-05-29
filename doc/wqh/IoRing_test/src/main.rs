mod lib;
use lib::*;

fn main() {
    let user_config = UserConfig::new();
    let mut global_config = FuseConfig::new();
    global_config.storage_max_conc_xmit = 64;

    user_config.init(global_config);

    let ui = UserInfo {
        uid: Uid(100),
        gid: Gid(1000),
    };

    // Set user config
    match user_config.set_config("usr.readonly", "true", &ui) {
        Ok(inode) => println!("Set config success: {:?}", inode.symlink.target),
        Err(e) => eprintln!("Error setting config: {}", e),
    }

    // Lookup config
    match user_config.lookup_config("usr.readonly", &ui) {
        Ok(inode) => println!("Lookup config: {}", inode.symlink.target),
        Err(e) => eprintln!("Error looking up config: {}", e),
    }

    // List all configs
    let (dirs, _) = user_config.list_config(&ui);
    for d in dirs {
        println!("Dir entry: {}", d.symlink.target);
    }
}