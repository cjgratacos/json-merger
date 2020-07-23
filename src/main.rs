use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use structopt::StructOpt;
use log::{Level, info, error, debug};
use simple_logger;

const OPEN_BRACKET: u8 = 91;
const CLOSE_BRACKET: u8 = 93;
const COMMA: u8 = 44;

#[derive(StructOpt, Debug)]
pub struct Cli {
    #[structopt(parse(from_os_str))]
    /// The path where the json collection lies
    pub path: PathBuf,
    /// Activate debug mode
    #[structopt(short, long)]
    pub debug: bool,
    /// Activate info mode
    #[structopt(short, long)]
    pub info: bool,
}

fn main() {
    let cli = Cli::from_args();

    // Setup logger
    match setup_logger(&cli) {
        Ok(_) =>{
            info!("Logger was successfully setup");
        },
        Err(s) => {
            error!("{}", s);
        }
    }

    let path =  Path::new(&cli.path);

    match validate_path(path) {
        Ok(_) => {
            info!("Path or sub-paths contains Json. [Path:{}]", path.to_str().unwrap());
            process(path);
        },
        Err(s) => {
            error!("{}", s);
        }
    };
}

fn process(path: &Path) {
    let p = path.to_str().unwrap();
   info!("Begin Processing path: {}", p);
   let mut files = Vec::new();
   for entry in fs::read_dir(path).unwrap() {
       let dir_entry = entry.unwrap();
       let entry_path = dir_entry.path();
       let entry_str = entry_path.to_str().unwrap();
       if entry_path.is_dir() {
           debug!("Path[{}] is a sub folder", entry_str);
           process(&entry_path);
       } else if entry_path.is_file() && entry_str.to_lowercase().ends_with(".json") {
           debug!("Path[{}] is a file", entry_str);
           files.push(entry_str.to_string());
       }
   }

   debug!("Path [{}] contains the following files: {:?}", p, files);

   info!("Found {} json files in path: {}", files.len(), p);
   if !files.is_empty() {
       let filename = &format!("{}/{}.json", path.to_str().unwrap(), path.file_name().unwrap().to_str().unwrap());

       debug!("Sorting files: {:?}", files);
       files.sort();
       debug!("Sorted files: {:?}", files);

       let mut out = File::create(filename).unwrap();

       out.write(&[OPEN_BRACKET]).unwrap();

       for (i, f) in files.iter().enumerate() {
           if f.eq(filename) {
               continue;
           }
           let mut file = File::open(f).unwrap();
           let mut buffer = Vec::new();

           file.read_to_end(&mut buffer).unwrap();

           // Cleanup first bracket
           if !buffer.is_empty() && buffer.first().unwrap().eq(&OPEN_BRACKET) {
               buffer.remove(0);
           }

           // CLeanup last bracket
           if !buffer.is_empty() && buffer.last().unwrap().eq(&CLOSE_BRACKET) {
               buffer.remove(buffer.len() -1);
           }

           if i < files.len() -1 {
               buffer.push(COMMA);
           }

           out.write_all(&buffer).unwrap();
       }

       out.write(&[CLOSE_BRACKET]).unwrap();
   }


   info!("Finished Processing Path: {}", path.to_str().unwrap());
}

fn validate_path(path: &Path) -> Result<(), String> {

    if !path.is_dir() {
        return Err("Path should be a directory".to_string());
    }

    if !contains_json_file(path) {
        return Err(format!("Path[{}] or sub-paths don't contain any Json files", path.to_str().unwrap()));
    }

    Ok(())
}

fn contains_json_file(path: &Path) -> bool {
    let mut result = false;

    let p = path.to_str().unwrap();
    info!("--> Begin searching path: {}", p);

    if path.is_dir() {
        info!("----> Path is directory: {}", p);
        for entry in fs::read_dir(path).unwrap() {
            let dir_entry = entry.unwrap();
            let entry_path = dir_entry.path();
            let entry_str = entry_path.to_str().unwrap();
            if entry_path.is_dir() {
                info!("------> Found sub directory: {}", entry_str);
                result = contains_json_file(&entry_path);

            } else if entry_path.is_file() {

                info!("------> Found file: {}", entry_str);

                if !result {
                    result = dir_entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .to_lowercase()
                        .ends_with(".json");
                }
            }
        }
    }

    info!("--> Finished searching path: {}. Found Json: {}", p, result);

    result
}

fn setup_logger(cli: &Cli) -> Result<(), String> {
    let mut level = Level::Warn;

    if cli.info {
        level = Level::Info;
    }

    if cli.debug {
        level = Level::Debug;
    }

    match simple_logger::init_with_level(level) {
        Ok(()) => Ok(()),
        Err(e) => Err(e.to_string())
    }
}
