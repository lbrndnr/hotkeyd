use std::fs::File;
use std::str::FromStr;
use std::{thread, time};
use std::process::Command;
use clap;
use daemonize::Daemonize;
use livesplit_hotkey::{Hook, Hotkey, KeyCode, Modifiers};
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Deserialize, Serialize, Debug)]
struct ProfileHotKey {
    keys: Vec<String>,
    modifiers: Vec<String>,
    command: String,
}

fn cli() -> clap::Command {
    clap::Command::new("hotkeyd")
        .about("Scripting at your fingertips")
        .subcommand_required(true)
        .arg_required_else_help(true)
        // .allow_external_subcommands(true)
        .subcommand(
            clap::Command::new("init")
                .about("Initializes the .hotkeyd file.")
                .arg_required_else_help(true),
        )
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn register_profile_hotkeys(hook: &Hook, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let payload = std::fs::read_to_string(path)?;
    let profile = serde_json::from_str::<Vec<ProfileHotKey>>(&payload)?;

    for phk in profile.iter() {
        let key = format!("Key{}", phk.keys[0].to_uppercase());        
        let mut vals: Vec<String> = phk.modifiers.iter().map(|m| capitalize(m)).collect();
        vals.push(key);
        println!("{}", vals.join("+").as_str());
        match Hotkey::from_str(vals.join("+").as_str()) {
            Ok(hotkey) => {
                let command = Box::new(phk.command.to_owned());
                hook.register(hotkey, move|| {
                    Command::new("sh")
                        .arg("-c")
                        .arg(command.as_ref())
                        .output()
                        .unwrap();
                })?
            }
            Err(()) => println!("Invalid payload")
        }
        
    }

    Ok(())
}

fn daemonize() {
    let stdout = File::create("/tmp/daemon.out").unwrap();
    let stderr = File::create("/tmp/daemon.err").unwrap();

    let daemonize = Daemonize::new()
        // .pid_file("/tmp/test.pid") // Every method except `new` and `start`
        // .chown_pid_file(true)      // is optional, see `Daemonize` documentation
        // .working_directory("/tmp") // for default behaviour.
        // .user("nobody")
        // .group("daemon") // Group name
        // .group(2)        // or group id.
        // .umask(0o777)    // Set umask, `0o027` by default.
        .stdout(stdout)  // Redirect stdout to `/tmp/daemon.out`.
        .stderr(stderr);  // Redirect stderr to `/tmp/daemon.err`.
        // .privileged_action(|| "Executed before drop privileges");

    // match daemonize.start() {
    //     Ok(_) => println!("Success, daemonized"),
    //     Err(e) => eprintln!("Error, {}", e),
    // }

}

fn main() {
    // let matches = cli().get_matches();

    // match matches.subcommand() {
    //     Some(("init", sub_matches)) => {
    //         init();
    //     }
    //     _ => unreachable!()
    // }

    // // Register the hotkey
    let hook = Hook::new().unwrap();

    let path = "/Users/Laurin/.hotkeyd";
    match register_profile_hotkeys(&hook, path) {
        Err(e) => eprintln!("Error, {}", e),
        _ => println!("OK")
    }

    loop {
        let latency = time::Duration::from_millis(1000);
        thread::sleep(latency);
    }
}
