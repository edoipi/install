#![crate_name = "install"]
#![feature(macro_rules)]
#![feature(plugin)]
#![allow(missing_copy_implementations)] //to be removed in later stage
#![allow(unused_variables)]  //to be removed in later stage 
#![feature(rustc_private)]
#![feature(collections)]
#![feature(associated_consts)]
/*#![feature(phase)]*/

/*
 * This file is part of the uutils coreutils package.
 *
 * (c) Matuesz Twaróg <implicent@gmail.com>
 *
 * For the full copyright and license information, please view the LICENSE file
 * that was distributed with this source code.
 */


#[macro_use] extern crate rustc_bitflags;
extern crate collections;
extern crate getopts;
extern crate regex;
extern crate log;
extern crate rustc;

use std::env::current_dir;
use std::ffi::OsString;
use std::path::PathBuf;
use std::boxed::Box;
use std::borrow::ToOwned;
use regex::Regex;
//use std::os::make_absolute;
//use rustc::util::fs::realpath;
use std::collections::HashSet;
use std::collections::HashMap;
use collections::string::String;
use collections::vec::Vec;
use getopts::Options;
use getopts::{
    //getopts,
    //optflag,
    //optopt,
    //OptGroup,
};
use std::fs;
use std::io;/*::{
    fs,
    FilePermission,
    FileType,
    GROUP_EXECUTE,
    GROUP_READ,
    GROUP_WRITE,
    OTHER_EXECUTE,
    OTHER_READ,
    OTHER_WRITE,
    USER_EXECUTE,
    USER_READ,
    USER_RWX,
    USER_WRITE,
};*/
use std::path::Path;
#[path = "../common/util.rs"]
#[macro_use]
mod util;

static NAME: &'static str = "install";

bitflags! {
    flags User: u32 {
        const USER  = 0x00000001,
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

enum Type {
    Add,
    Remove,
    Set,
}

/*struct Action {
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
}*/

pub fn uumain(args: Vec<String>) -> i32 {
    let program = args[0].clone();
    let mut opts = Options::new();
    
    opts.optflag("h", "help", "display this help and exit");
    opts.optflag("v", "version", "output version information and exit");
    opts.optopt("t", "target-directory", "Specify the destination directory", "");
    opts.optopt("m", "mode", "Set the file mode bits for the installed file or directory to mode", "");
    
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            //crash!(1, "{}", e);
            panic!(e.to_string());
        },
    };
    
    if matches.opt_present("help") {
        print_usage(opts);
        return 0;
    }
    
    if matches.opt_present("version") {
        print_version();
        return 0;
    }
    
    let mut free = matches.free.clone();
    
    let dest_str: String = match matches.opt_str("target-directory") {
        Some(x) => x.to_owned(),
        None => {
            if free.len() <= 1 {
                show_error!("Missing TARGET argument.  Try --help.");
                return 1;
            } else {
                let tmp = free.pop();
                tmp.unwrap()
            }
        },
    };

    let dest = Path::new(&dest_str);
    let sources: Vec<Box<&Path>> = if free.len() <= 0 {
        println!("Missing SOURCE argument. Try --help.");
        show_error!("Missing SOURCE argument. Try --help.");
        return 1;
    } else {
        let mut tmp : Vec<Box<&Path>> = Vec::new();
        for i in 0..free.len() {
            if fs::metadata(Path::new(&free[i].clone())).is_err() {
                println!("cannot stat ‘{}’: No such file or directory {}", free[i], fs::metadata(Path::new(&free[i].clone())).is_err());
                show_error!("cannot stat ‘{}’: No such file or directory", free[i]);
                return 1;
            }
            let boxer = Box::new(Path::new(&free[i]));
            tmp.push(boxer);
        }
        tmp
    };
    
    let mode = match matches.opt_str("mode") {
        Some(x) => parse_mode(x),
        None => 755,
    };
    
    let is_dest_dir = match fs::metadata(&dest) {
        Ok(m) => m.is_dir(),
        Err(_) => false
    };
    
    println!("is dest dir {}", is_dest_dir);
    
    if matches.opt_present("target-directory") || sources.len() > 1  || is_dest_dir {
        println!("many files");
        files_to_directory(sources, dest, mode);
    } else {
        println!("one file {} {}", (*sources[0]).display(), dest.display());
        file_to_file(&*sources[0], dest, mode);
    }
    0
}

fn file_to_file(source: &Path, dest: &Path, mode: i32) {
    let real_source = real(source);
    let real_dest = real(dest);
    
    println!("realll {:?} {:?} {}", real_source, real_dest, real_source==real_dest);
    
    if real_source == real_dest {
        println!("{0} and {1} are the same file", source.display(), dest.display());
        panic!();
        //crash!(1, "{0} and {1} are the same file", source.display(), dest.display());
    }
    
    match fs::copy(source, dest) {
        Ok(_) => (),
        Err(e) => {
            //crash!(1, "{}", e);
            panic!(e.to_string());
        },
    }
    
    /*match fs::set_permissions(&dest, mode) {
        Ok(m) => m,
        Err(e) => {
            //crash!(1, "{}", e);
            panic!(e.to_string());
        },
    }*/
}

fn files_to_directory(sources : Vec<Box<&Path>>, dest : &Path, mode: i32) {
    match fs::metadata(&dest) {
        Ok(m) => if !m.is_dir() {
            crash!(1, "failed to access ‘{}’: No such file or directory", dest.to_str());
        },
        Err(_) => {
            crash!(1, "target ‘{}’ is not a directory", dest.display());
        }
    };
    
    let mut set = HashSet::new();
    
    for i in 0..sources.len() {
        let mut stat = fs::metadata(&*sources[i]);
        if stat.is_ok() && stat.unwrap().is_dir() {
            println!("install: omitting directory ‘{}’", sources[i].display());
            continue;
        }
        let mut tmp_dest_buf = dest.to_path_buf().clone().to_owned();
        tmp_dest_buf.push(match sources[i].file_name() {
            Some(m) => m,
            None => unreachable!(),
        });
        let mut tmp_dest : &Path = tmp_dest_buf.as_path();
        stat = fs::metadata(&tmp_dest);
        if stat.is_ok() && stat.unwrap().is_dir() {
            println!("install: cannot overwrite directory ‘{}’ with non-directory", tmp_dest.display());
            continue;
        }
        
        let real_source = real(&tmp_dest);
        let real_dest = real(&*sources[i]);
        
        println!("realll  {:?}   {:?}", real_source, real_dest);
        
        if real_source == real_dest {
            println!("install: {0} and {1} are the same file", sources[i].display(), tmp_dest.display());
            continue;
        }
        
        if set.contains(&real_dest){
            println!("install: will not overwrite just-created ‘{}’ with ‘{}’", tmp_dest.display(), sources[i].display());
            continue;
        }
        
        match fs::copy(&*sources[i], &*tmp_dest) {
            Ok(m) => {
                set.insert(real_dest);
                m
            },
            Err(e) => {
                //crash!(1, "{}", e);
                panic!(e.to_string());
            },
        };
        
        /*match fs::set_permissions(&tmp_dest, mode) {
            Ok(m) => m,
            Err(e) => {
                //crash!(1, "{}", e);
                panic!(e.to_string());
            },
        }*/
    }
}

fn parse_mode(s : String) -> i32 {
    /*let mut map = HashMap::new();
    map.insert((USER, READ), USER_READ);
    map.insert((USER, WRITE), USER_WRITE);
    map.insert((USER, EXECUTE), USER_EXECUTE);
    map.insert((GROUP, READ), GROUP_READ);
    map.insert((GROUP, WRITE), GROUP_WRITE);
    map.insert((GROUP, EXECUTE), GROUP_EXECUTE);
    map.insert((OTHER, READ), OTHER_READ);
    map.insert((OTHER, WRITE), OTHER_WRITE);
    map.insert((OTHER, EXECUTE), OTHER_EXECUTE);
    
    let mut out = FilePermission::empty();
    let split: Vec<&str> = s.as_slice().split(',').collect();
    let regexp = Regex::new(r"^[ugoa]*[-=+][rwx]*$").unwrap(); /////////////////////tutaj
    for i in split.iter() {
    
        if !regexp.is_match(i.as_slice()) {
            crash!(1, "invalid mode '{}'", s);
        }
        
        let mut user = User::empty();
        let mut permission = Permission::empty();
        let re = Regex::new(r"[-=+]").unwrap();//////////////////////////////////tutaj
        let sp: Vec<&str> = re.split(i.as_slice()).collect();
        for c in sp[0].chars() {
            user = user | match c {
                'u' => USER,
                'g' => GROUP,
                'o' => OTHER,
                'a' => User::all(),
                _   => unreachable!(),
            };
        }
        for c in sp[1].chars() {
            permission = permission | match c {
                'r' => READ,
                'w' => WRITE,
                'x' => EXECUTE,
                _   => unreachable!(),
            };
        }
        
        let mut file_permissions = FilePermission::empty();
        
        for u in vec![USER, GROUP, OTHER].into_iter() {
            if u & user != User::empty() {
                for p in vec![READ, WRITE, EXECUTE].into_iter() {
                    if p & permission != Permission::empty() {
                        file_permissions.insert(*map.get(&(u.clone(), p.clone())).unwrap());
                    }
                }
            }
        }
        
        let mut cap = match re.captures(i.as_slice()) {
            Some(s) => s.at(0).chars(),
            None => unreachable!(),
        };
        
        let operator = match cap.next() {
            Some(s) => match s {
                '-' => Type::Remove,
                '=' => Type::Set,
                '+' => Type::Add,
                _   => unreachable!(),
            },
            None => unreachable!(),
        };
        
        let action = Action{ t: operator, p: file_permissions };
        action.apply_on(&mut out);
    }*/
    755
}

fn real(path: & Path) -> Box<PathBuf> {
    let mut real_path = current_dir().unwrap();
    
    for component in path.components() {
    	let mut real_path_clone = real_path.clone();
    	real_path_clone.push(component.as_os_str());
    	
    	let next : OsString = match fs::read_link(&real_path_clone) {
    	    Ok(m) => {  println!("here");
    	                m.file_name().unwrap().to_owned()},
    	    Err(e) => (*component.as_os_str()).to_os_string()  
    	};
    	real_path.push(next);
    	println!("pav {} {}", real_path.display(), real_path_clone.display());
    }

    let bbox = Box::new(Path::new(real_path.as_path()).to_owned());
    bbox
}

fn print_usage(opts: Options) {
    let msg = format!("Usage: install [OPTION]... [-T] SOURCE DEST
  or:  install [OPTION]... SOURCE... DIRECTORY
  or:  install [OPTION]... -t DIRECTORY SOURCE...
  or:  install [OPTION]... -d DIRECTORY...
    ");
    println!("{}",  opts.usage(&msg));
}

fn print_version() {
	println!("install version 1.0.0");
}
