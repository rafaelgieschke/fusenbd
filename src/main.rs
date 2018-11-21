#![allow(unused, dead_code)]

#[macro_use]
extern crate structopt;
extern crate fuse;
extern crate nbd;
extern crate readwriteseekfs;
extern crate bufstream;
extern crate rand;

use std::process::Command;
use rand::{thread_rng, Rng};
use std::time::Duration;
use std::thread::sleep;

use fuse::Filesystem;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use structopt::StructOpt;

use std::fs::File;
use std::io::{Error, ErrorKind, Result};

use std::net::TcpStream;
use std::os::unix::net::UnixStream;

use nbd::client::{handshake, NbdClient};
use readwriteseekfs::{ReadSeekFs,ReadWriteSeekFs};

#[derive(StructOpt, Debug)]
#[structopt(
    after_help = "
Example:
    fusenbd nbd.dat image.qcow -f qcow2
    
    fusenbd -r sda1 image.qcow -f qcow2 -- -o allow_empty,ro,fsname=qwerty,auto_unmount
",
)]
struct Opt {
    /// Regular file to use as mountpoint
    #[structopt(parse(from_os_str))]
    file: PathBuf,
    /// Path to image
    image: String,
    /// Named export to use.
    #[structopt(short = "-x", long = "export-name", default_value = "")]
    export: String,

    /// Image format (see qemu-nbd; e.g., raw, qcow2, ...)
    #[structopt(short = "f", long = "format", default_value = "")]
    format: String,

    /// Mount read-only
    #[structopt(short = "r", long = "read-only")]
    ro: bool,

    /// The rest of FUSE options.
    #[structopt(parse(from_os_str))]
    opts: Vec<OsString>,
}

fn temp_path() -> PathBuf {
    let tmp: String = thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(50)
        .collect();
    std::env::temp_dir().join(tmp)
}

fn run() -> Result<()> {
    let mut qemunbd_process;
let res = {
    let mut cmd = Opt::from_args();

    let socket_path = temp_path();
    let mut qemunbd = Command::new("qemu-nbd");
    qemunbd.arg("-k").arg(&socket_path);
    if cmd.export != "" {
        qemunbd.arg("-x").arg(&cmd.export);
    }
    if cmd.format != "" {
        qemunbd.arg("-f").arg(cmd.format);
    }
    qemunbd.arg(cmd.image);

    qemunbd_process = qemunbd.spawn()?;

    while socket_path.metadata().is_err() {
        sleep(Duration::from_millis(100));
    }

    match cmd.file.metadata() {
        Ok(ref m) if m.is_dir() => eprintln!("Warning: {:?} is a directory, not a file", cmd.file),
        Ok(ref m) if m.is_file() => (),
        Ok(_) => eprintln!("Warning: can't determine type of {:?}", cmd.file),
        Err(ref e) if e.kind() == ErrorKind::NotFound => {
            drop(File::create(cmd.file.clone()));
        }
        Err(e) => Err(e)?,
    }

    let mut tcp = UnixStream::connect(&socket_path)?;
    let mut tcp = bufstream::BufStream::new(tcp);
    let export = handshake(&mut tcp, cmd.export.as_bytes())?;
    let mut client = NbdClient::new(&mut tcp, &export);

    let opts: Vec<&OsStr> = cmd.opts.iter().map(AsRef::as_ref).collect();
    
    if cmd.ro {
        let fs = readwriteseekfs::ReadSeekFs::new(client, 1024)?;
        fuse::mount(fs, &cmd.file.as_path(), opts.as_slice())
    } else {
        let fs = readwriteseekfs::ReadWriteSeekFs::new(client, 1024)?;
        fuse::mount(fs, &cmd.file.as_path(), opts.as_slice())
    }
};
    qemunbd_process.wait();
    res
}

fn main() {
    let r = run();

    if let Err(e) = r {
        eprintln!("fusenbd: {}", e);
        ::std::process::exit(1);
    }
}
