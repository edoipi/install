
fn decode(f: FilePermission) { // for debug only
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
