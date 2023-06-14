use std::fs::File;
use std::str::FromStr;
use std::io::{Error, ErrorKind};
use std::thread;
use std::process::{Command, exit};
use clap;
use daemonize::Daemonize;
use livesplit_hotkey::{permission, Hook, Hotkey};
use serde::{Serialize, Deserialize};
use serde_json;

type BoxResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Deserialize, Serialize, Debug)]
struct ProfileHotKey {
    keys: Vec<String>,
    modifiers: Vec<String>,
    command: String,
}

fn cli() -> clap::Command {
    clap::Command::new("hotkeyd")
        .about("Scripting at your fingertips")
        .subcommand(
            clap::Command::new("lint")
                .about("Lint the .hotkey file.")
        )
}

fn new_hotkey(phk: &ProfileHotKey) -> Result<Hotkey, ()> {
    fn capitalize(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
    
    let key = format!("Key{}", phk.keys[0].to_uppercase());        
    let mut vals: Vec<String> = phk.modifiers.iter().map(|m| capitalize(m)).collect();
    vals.push(key);

    Hotkey::from_str(vals.join("+").as_str())
}

fn register_profile_hotkeys(hook: &Hook, path: &str) -> BoxResult<()> {
    let payload = std::fs::read_to_string(path)?;
    let profile = serde_json::from_str::<Vec<ProfileHotKey>>(&payload)?;

    for phk in profile.iter() {
        match new_hotkey(phk) {
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
            Err(()) => {
                return Err(Box::new(Error::new(ErrorKind::InvalidData, "Unknown key and/or modifiers specified.")))
            }
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

    match daemonize.start() {
        Ok(_) => println!("Success, daemonized"),
        Err(e) => eprintln!("Error, {}", e),
    }

}

fn main() {
    let matches = cli().get_matches();

    if !permission::request() {
        return;
    }
    // if !permission::request() {
    //     eprintln!("Accessibility permission was not granted. Terminating.");
    //     if let Err(err) = open::that("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility") {
    //         eprintln!("{}", err);
    //     }
    //     return;
    // }

    let hook = match Hook::new() {
        Ok(hook) => hook,
        Err(err) => {
            eprintln!("Error: {} Please open an issue at https://github.com/lbrndnr/hotkeyd.", err);
            exit(1);
        }
    };
    let path = "/Users/Laurin/.hotkeyd";
    let res = register_profile_hotkeys(&hook, path);

    if let Err(e) = res {
        eprintln!("Error, {}", e);
        return
    }

    match matches.subcommand() {
        Some(("lint", _)) => {
            println!("Looking good!");
            return
        }
        _ => {
            loop {
                thread::park();
            }
        }
    }
}
