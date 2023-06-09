use std::fs::File;
use std::{thread, time};
use std::process::Command;
use clap;
use daemonize::Daemonize;
use livesplit_hotkey::{Hook, KeyCode, Modifiers};

fn cli() -> clap::Command {
    clap::Command::new("hotkeyd")
        .about("Scripting at your fingertips")
        .subcommand_required(true)
        .arg_required_else_help(true)
        // .allow_external_subcommands(true)
        .subcommand(
            clap::Command::new("detach")
                .about("Clones repos")
                // .arg(arg!(<REMOTE> "The remote to clone"))
                .arg_required_else_help(true),
        )
}

fn main() {
    // let matches = cli().get_matches();

    // match matches.subcommand() {
    //     Some(("detach", sub_matches)) => {
    //         println!("detach")
    //     }
    //     _ => unreachable!()
    // }

    println!("OOOOOK");

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

    // Register the hotkey
    let hook = Hook::new().unwrap();

    hook.register(KeyCode::KeyK.with_modifiers(Modifiers::ALT), || {
        Command::new("sh")
            .arg("-c")
            .arg(r"open /Applications/Visual\ Studio\ Code.app")
            .output()
            .unwrap();
    })
    .unwrap();

    loop {
        let latency = time::Duration::from_millis(500);
        thread::sleep(latency);
    }
}
