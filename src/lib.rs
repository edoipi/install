#![crate_name = "install"]
#![feature(macro_rules)]
#![feature(phase)]

#![allow(missing_copy_implementations)] //
#![allow(unused_variables)]  // 
#![allow(unused_imports)]  // 

extern crate collections;
extern crate getopts;
#[phase(plugin, link)] extern crate log;
extern crate regex;
#[phase(plugin)] extern crate regex_macros;
extern crate rustc;

use std::iter::range_step;
use std::fmt;
use std::collections::HashSet;
use std::collections::HashMap;
use collections::string::String;
use collections::vec::Vec;
use getopts::{
    getopts,
    optflag,
    optopt,
};
use regex::Regex;
use rustc::util::fs::realpath;
use std::io::{
    fs,
    FilePermission,
    FileType,
    GROUP_EXECUTE,
    GROUP_READ,
    GROUP_RWX,
    GROUP_WRITE,
    OTHER_EXECUTE,
    OTHER_READ,
    OTHER_RWX,
    OTHER_WRITE,
    USER_EXECUTE,
    USER_READ,
    USER_RWX,
    USER_WRITE,
};

bitflags! {
    flags User: u32 {
        const OWNER = 0x00000001,
        const GROUP = 0x00000010,
        const OTHER = 0x00000100,
    }
}

bitflags! {
    flags Permission: u32 {
        const READ    = 0x00000001,
        const WRITE   = 0x00000010,
        const EXECUTE = 0x00000100,
    }
}

fn decode(f: FilePermission) {
    let mut map = HashMap::new();
    map.insert(GROUP_EXECUTE, "GROUP_EXECUTE");
    map.insert(GROUP_READ, "GROUP_READ");
    map.insert(GROUP_WRITE, "GROUP_WRITE");
    map.insert(OTHER_EXECUTE, "OTHER_EXECUTE");
    map.insert(OTHER_READ, "OTHER_READ");
    map.insert(OTHER_WRITE, "OTHER_WRITE");
    map.insert(USER_EXECUTE, "USER_EXECUTE");
    map.insert(USER_READ, "USER_READ");
    map.insert(USER_WRITE, "USER_WRITE");
    for &p in [GROUP_EXECUTE,
                GROUP_READ,
                GROUP_WRITE,
                OTHER_EXECUTE,
                OTHER_READ,
                OTHER_WRITE,
                USER_EXECUTE,
                USER_READ,
                USER_WRITE].iter() {
        if f & p != FilePermission::empty() {
            print!("{} ", map.get(&p).unwrap());
        }
    }
}

enum Type {
    Add,
    Remove,
    Set,
}

struct Action {
    t: Type,
    p: FilePermission,
}

impl Action {
    fn apply_on(&self, p: &mut FilePermission) {
        match self.t {
            Type::Add => p.insert(self.p),
            Type::Remove => p.remove(self.p),
            Type::Set => {
                p.remove(FilePermission::all());
                p.insert(self.p)
            },
        }
    }
}

pub fn uumain(args: Vec<String>) -> int {
    //let args = os::args();
    let program = args[0].clone();
    let opts = [
        optflag("h", "help", "display this help and exit"),
        optflag("v", "version", "output version information and exit"),
        optopt("t", "target-directory", "Specify the destination directory", ""),
        optopt("m", "mode", "Set the file mode bits for the installed file or directory to mode", ""),
    ];
    let matches = match getopts(args.tail(), &opts) {
        Ok(m) => m,
        Err(e) => {
            error!("error: {}", e);
            panic!()
        },
    };
    
    let mut free = matches.free.clone();
    
    let mode = match matches.opt_str("mode") {
        Some(x) => parse_mode(x),
        None => {
            let mut v = Vec::new();
            v.push(Action{t: Type::Set, p: USER_RWX|GROUP_READ|GROUP_EXECUTE|OTHER_READ|OTHER_EXECUTE});
            v
        },
    };
    /*for m in mode.iter() {
        decode(m.p);
        println!("");
    }
    return 0;*/
    
    let dest : Path = match matches.opt_str("target-directory") {
        Some(x) => Path::new(x),
        None => {
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
        },
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
        Ok(m) => m.kind == std::io::FileType::Directory,
        Err(_) => false
    };
    
    if matches.opt_present("target-directory") || sources.len()>1  || is_dest_dir {
        files_to_directory(sources, dest, mode);
    } else {
        file_to_file(sources[0].clone(), dest, mode);
    }
    0
}

fn file_to_file(source: Path, dest: Path, mode: Vec<Action>) {
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
    }
    
    let mut current_perm = match fs::stat(&dest) {
        Ok(stat) => stat.perm,
        Err(e) => {error!("error: {}", e);
            panic!()
        },
    };
    
    for m in mode.iter() {
        m.apply_on(&mut current_perm);
    }
    //decode(current_perm);
    match fs::chmod(&dest, current_perm) {
        Ok(m) => m,
        Err(e) => {
            error!("error: {}", e);
            panic!()
        },
    }
}

fn files_to_directory(sources : Vec<Path>, dest : Path, mode: Vec<Action>) {
    match fs::stat(&dest) {
        Ok(m) => if m.kind != FileType::Directory {
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
        if stat.is_ok() && stat.unwrap().kind == FileType::Directory {
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
        if stat.is_ok() && stat.unwrap().kind == FileType::Directory {
            println!("install: cannot overwrite directory ‘{}’ with non-directory", tmp_dest.display());
            continue;
        }
        //TO DO: make sure not ot overrite file with itself
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
        
        let mut current_perm = match fs::stat(&tmp_dest) {
            Ok(stat) => stat.perm,
            Err(e) => {error!("error: {}", e);
                panic!()
            },
        };
        
        for m in mode.iter() {
            m.apply_on(&mut current_perm);
        }
        //decode(current_perm);
        match fs::chmod(&tmp_dest, current_perm) {
            Ok(m) => m,
            Err(e) => {
                error!("error: {}", e);
                panic!()
            },
        }
    }
}

fn parse_mode(s : String) -> Vec<Action> {
    //println!("OWNER: {}, GROUP: {}, OTHER: {}, ALL: {}", OWNER, GROUP, OTHER, User::all());
    let mut map = HashMap::new();
    map.insert((OWNER, READ), USER_READ);
    map.insert((OWNER, WRITE), USER_WRITE);
    map.insert((OWNER, EXECUTE), USER_EXECUTE);
    map.insert((GROUP, READ), GROUP_READ);
    map.insert((GROUP, WRITE), GROUP_WRITE);
    map.insert((GROUP, EXECUTE), GROUP_EXECUTE);
    map.insert((OTHER, READ), OTHER_READ);
    map.insert((OTHER, WRITE), OTHER_WRITE);
    map.insert((OTHER, EXECUTE), OTHER_EXECUTE);
    
    let mut out: Vec<Action> = Vec::new();
    let split: Vec<&str> = s.as_slice().split(',').collect();
    let regexp = regex!(r"^[ugoa]*[-=+][rwx]*$");
    for i in split.iter() {
    
        if !regexp.is_match(i.as_slice()) {
            error!("invalid mode ‘{}’", s);
            panic!()
        }
        
        let mut user = User::empty();
        let mut permission = Permission::empty();
        let re = regex!(r"[-=+]");
        let sp: Vec<&str> = re.split(i.as_slice()).collect();
        for c in sp[0].chars() {
            user = user | match c {
                'u' => OWNER,
                'g' => GROUP,
                'o' => OTHER,
                'a' => User::all(),
                _   => panic!(),
            };
        }
        for c in sp[1].chars() {
            permission = permission | match c {
                'r' => READ,
                'w' => WRITE,
                'x' => EXECUTE,
                _   => panic!(),
            };
        }
        //user = user&user;
        let mut file_permissions = FilePermission::empty();
        
        for u in [OWNER, GROUP, OTHER].iter() {
            if *u & user != User::empty() {
                for p in [READ, WRITE, EXECUTE].iter() {
                    if *p & permission != Permission::empty() {
                        //let g = (*u, *p);
                        file_permissions.insert( match map.get(&((*u).clone(), (*p).clone())) {
                            Some(s) => *s,
                            None => panic!(),
                        } );
                    }
                }
            }
        }
        let mut cap= match re.captures(i.as_slice()) {
            Some(s) => s.at(0).chars(),
            None => panic!(),
        };
        
        let operator = match cap.next() {
            Some(s) => match s {
                '-' => Type::Remove,
                '=' => Type::Set,
                '+' => Type::Add,
                _   => panic!(),
            },
            None => panic!(),
        };
        out.push(Action{t: operator, p: file_permissions});
    }
    out
}