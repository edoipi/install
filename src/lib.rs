#![crate_name = "install"]
#![feature(macro_rules)]
#![feature(phase)]

extern crate collections;
extern crate getopts;
#[phase(plugin, link)] extern crate log;
extern crate rustc;

use std::collections::HashSet;
use collections::string::String;
use collections::vec::Vec;
use getopts::{
    getopts,
    optflag,
    optopt,
};
use rustc::util::fs::realpath;
use std::io::fs;

pub fn uumain(args: Vec<String>) -> int {
    //let args = os::args();
    let program = args[0].clone();
    let opts = [
        optflag("h", "help", "display this help and exit"),
        optflag("v", "version", "output version information and exit"),
        optopt("t", "target-directory", "Specify the destination directory", ""),
    ];
    let matches = match getopts(args.tail(), opts) {
        Ok(m) => m,
        Err(e) => {
            error!("error: {}", e);
            panic!()
        },
    };
    
    let mut free = matches.free.clone();
    
    let dest : Path = if matches.opt_present("target-directory") {
        match matches.opt_str("t") {
            Some(x) => Path::new(x),
            None => {
                error!("error: Missing TARGET argument. Try --help.");
                panic!()
            },
        }
    } else {
        match free.len() {
            0...1 => {
                error!("error: Missing TARGET argument. Try --help.");
                panic!()
            },
            _ => {
                let tmp = free.pop();
                Path::new(tmp.unwrap())
            }
        }
    };
    let sources : Vec<Path> = match free.len() {
        0 => {
            error!("error: Missing SOURCE argument. Try --help.");
            panic!()
        },
        _ => {
            let mut tmp : Vec<Path> = Vec::new();
            for i in range (0, free.len()) {
                if fs::stat(&Path::new(free[i].clone())).is_err() {
                    error!("cannot stat ‘{}’: No such file or directory", free[i]);
                    panic!()
                }
                tmp.push( Path::new(free[i].clone()) );
            }
            tmp
        }
    };
    
    let is_dest_dir = match fs::stat(&dest) {
        Ok(m) => m.kind == std::io::FileType::TypeDirectory,
        Err(_) => false
    };
    
    if matches.opt_present("target-directory") || sources.len()>1  || is_dest_dir {
        files_to_directory(sources, dest);
    } else {
        file_to_file(sources[0].clone(), dest);
    }
    0
}

fn file_to_file(source : Path, dest : Path) {
    let real_source = match realpath(&source) {
        Ok(m) => m,
        Err(e) => {
            error!("error: {}", e);
            panic!()
        },
    };
    let real_dest = match realpath(&dest) {
        Ok(m) => m,
        Err(e) => {
            error!("error: {}", e);
            panic!()
        },
    };
    if real_source==real_dest {
        error!("error: {0} and {1} are the same file", source.display(), dest.display());
        panic!()
    }
    
    match fs::copy(&source, &dest) {
        Ok(m) => m,
        Err(e) => {
            error!("error: {}", e);
            panic!()
        },
    };
}

fn files_to_directory(sources : Vec<Path>, dest : Path) {
    match fs::stat(&dest) {
        Ok(m) => if m.kind!=std::io::FileType::TypeDirectory {
                error!("failed to access ‘{}’: No such file or directory", dest.display());
                panic!()
            },
        Err(_) => {
            error!("target ‘{}’ is not a directory", dest.display());
            panic!()
        }
    };
    
    let mut set = HashSet::new();
    
    for i in range (0, sources.len()) {
        let mut stat = fs::stat(&sources[i]);
        if stat.is_ok() && stat.unwrap().kind == std::io::FileType::TypeDirectory {
            println!("install: omitting directory ‘{}’", sources[i].display());
            continue;
        }
        let mut tmp_dest = dest.clone();
        tmp_dest.push( match sources[i].filename_str() {
            Some(m) => m,
            None => {
                error!("error");
                panic!()
            },
        });
        
        stat = fs::stat(&tmp_dest);
        if stat.is_ok() && stat.unwrap().kind == std::io::FileType::TypeDirectory {
            println!("install: cannot overwrite directory ‘{}’ with non-directory", tmp_dest.display());
            continue;
        }
        
        let real_dest = match realpath(&tmp_dest) {
            Ok(m) => m,
            Err(e) => {
                error!("error: {}", e);
                panic!()
            },
        };
        
        if set.contains(&real_dest){
            println!("install: will not overwrite just-created ‘{}’ with ‘{}’", tmp_dest.display(), sources[i].display());
            continue;
        }
        
        match fs::copy(&sources[i], &tmp_dest) {
            Ok(m) => {
                set.insert(real_dest);
                m
                },
            Err(e) => {
                error!("error: {}", e);
                panic!()
            },
        };
    }
}
