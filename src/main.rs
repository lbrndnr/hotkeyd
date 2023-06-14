use auto_launch::AutoLaunch;
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;
use std::io::{Error, ErrorKind};
use std::thread;
use std::process::{Command, exit};
use clap;
use home;
use daemonize::Daemonize;
use livesplit_hotkey::{permission, Hook, Hotkey};
use serde::{Serialize, Deserialize};
use serde_json;
use anyhow::{Result, Context};

#[derive(Deserialize, Serialize, Debug)]
struct ProfileHotKey {
    key: String,
    modifiers: Vec<String>,
    command: String,
}

fn cli() -> clap::Command {
    clap::Command::new("hotkeyd")
        .about("Simple and hackable global keyboard shortcuts.")
        .subcommand(
            clap::Command::new("lint")
                .about("Lint the .hotkey file.")
        )
        .subcommand(
            clap::Command::new("setup")
                .about("Setup for daily usage.")
        )
}

fn new_hotkey(phk: &ProfileHotKey) -> Result<Hotkey> {
    fn capitalize(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
    
    let key = format!("Key{}", phk.key.to_uppercase());        
    let mut vals: Vec<String> = phk.modifiers.iter().map(|m| capitalize(m)).collect();
    vals.push(key);

    Hotkey::from_str(vals.join("+").as_str())
        .map_err(|_| anyhow::Error::new(Error::new(ErrorKind::InvalidData, "Unknown key and/or modifiers specified.")))
}

fn register_profile_hotkeys(hook: &Hook, path: &PathBuf) -> Result<()> {
    let payload = std::fs::read_to_string(path)?;
    let profile = serde_json::from_str::<Vec<ProfileHotKey>>(&payload)?;

    for phk in profile.iter() {
        let hotkey = new_hotkey(phk)?;
        let command = Box::new(phk.command.to_owned());
        hook.register(hotkey, move|| {
            Command::new("sh")
                .arg("-c")
                .arg(command.as_ref())
                .output()
                .unwrap();
        })?;
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

    match daemonize.start() {
        Ok(_) => println!("Success, daemonized"),
        Err(e) => eprintln!("Error, {}", e),
    }

}

fn setup(path: &PathBuf) -> Result<()> {
    auto_launch()?;
    request_permission();

    Ok(())
}

fn auto_launch() -> Result<()> {
    let bin_path = std::env::current_exe()?;
    if let Some(bin_path) = bin_path.to_str() {
        let auto = AutoLaunch::new("hotkeyd", bin_path, true, &[] as &[&str]);
        auto.enable().context("Failed to register hotkeyd as launch item.")
    }
    else {
        Err(Error::new(ErrorKind::NotFound, "Did not find the current binary."))
            .context("Could not convert the binary path to a string.")
    }
}

fn request_permission() {
    if !permission::request() {
        eprintln!("hotkeyd needs your permission to observe the keyboard. 
                   Grant the permission and run the command again. 
                   You might have to restart your terminal.");
        exit(1);
    }
}

fn main() {
    let matches = cli().get_matches();
    let subcommand = matches.subcommand();

    let home_dir = match home::home_dir() {
        Some(dir) => dir,
        None => {
            eprintln!("Error: Did not .hotkeyd profile. Please open an issue at https://github.com/lbrndnr/hotkeyd.");
            exit(2);
        }
    };

    let profile_path = home_dir.join(".hotkeyd");

    // Accessibility permissions are needed before registering the hook
    if let Some(("setup", _)) = subcommand {
        if let Err(err) = setup(&profile_path) {
            eprintln!("Error: {} Please open an issue at https://github.com/lbrndnr/hotkeyd.", err);
        }
        exit(0)
    }

    request_permission();

    let hook = match Hook::new() {
        Ok(hook) => hook,
        Err(err) => {
            eprintln!("Error: {} Please open an issue at https://github.com/lbrndnr/hotkeyd.", err);
            exit(3);
        }
    };

    if let Err(err) = register_profile_hotkeys(&hook, &profile_path) {
        eprintln!("Error, {}", err);
        exit(4);
    }

    match matches.subcommand() {
        Some(("setup", _)) => unreachable!(),
        Some(("lint", _)) => println!("Looking good!"),
        _ => {
            loop {
                thread::park();
            }
        }
    }
}
