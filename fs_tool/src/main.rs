mod rfs;
mod uninit_cell;

use clap::{App, Arg};
use rfs::block_cache;
use rfs::block_dev::BlockDevice;
use rfs::rfs::RustedFileSystem;
use rfs::BLOCK_SZ;
use std::fs::{read_dir, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::rc::Rc;
use std::sync::Mutex;

struct BlockFile(Mutex<File>);

impl BlockDevice for BlockFile {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.read(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.write(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
    }
}

fn config() -> (String, String, u64) {
    let matches = App::new("RustedFileSystem packer")
        .arg(
            Arg::with_name("source")
                .short("s")
                .long("source")
                .takes_value(true)
                .help("Executable source dir"),
        )
        .arg(
            Arg::with_name("target")
                .short("t")
                .long("target")
                .takes_value(true)
                .help("Executable target dir"),
        )
        .arg(
            Arg::with_name("blocks")
                .short("b")
                .long("blocks")
                .takes_value(true)
                .help("FileSystem blocks"),
        )
        .get_matches();
    let src_path = matches.value_of("source").expect("no src");
    let target_path = matches.value_of("target").expect("no dst");
    let blocks = matches
        .value_of("blocks")
        .expect("no blocks")
        .parse::<u64>()
        .unwrap();
    println!("src_path = {}", src_path);
    println!("target_path = {}", target_path);
    (src_path.to_owned(), target_path.to_owned(), blocks)
}

fn main() {
    let (src_path, target_path, blocks) = config();
    let block_file = Rc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(format!("{}/{}", target_path, "fs.img"))
            .unwrap();
        f.set_len(blocks * BLOCK_SZ as u64).unwrap();
        f
    })));

    block_cache::init();
    let rfs = RustedFileSystem::format(block_file, blocks as u32, 1);
    let root_inode = Rc::new(RustedFileSystem::root_inode(&rfs));
    root_inode.set_default_dirent(root_inode.get_inode_id());
    let bin_inode = root_inode
        .create("bin", rfs::layout::InodeType::Directory)
        .unwrap();
    bin_inode.set_default_dirent(root_inode.get_inode_id());

    let apps: Vec<_> = read_dir(src_path.clone())
        .unwrap()
        .into_iter()
        .filter(|dir_entry| dir_entry.as_ref().unwrap().file_type().unwrap().is_file())
        .map(|dir_entry| dir_entry.unwrap().file_name().into_string().unwrap())
        .filter(|s| s.find(".").is_none())
        .collect();
    for app in apps {
        let mut host_file = File::open(format!("{}/{}", src_path, app)).unwrap();
        let mut all_data: Vec<u8> = Vec::new();
        host_file.read_to_end(&mut all_data).unwrap();
        let inode = bin_inode
            .create(app.as_str(), rfs::layout::InodeType::File)
            .unwrap();
        inode.write_at(0, all_data.as_slice());
    }
}
