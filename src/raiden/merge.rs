use std::io::File;
use std::io::Reader;
use std::cmp::min;
use super::types::{CHUNK_SIZE, Chunk};

fn open_disks(source_filename: &str, disks: usize) -> (Vec<Option<File>>, Option<usize>, usize) {
    let mut optional_disk_files: Vec<Option<File>> = Vec::with_capacity(disks as usize);
    let mut amount_missing = 0;
    let mut missing_index = None;
    let mut file_length: Option<u64> = None;

    for s in range(0, disks) {
        let path = Path::new(format!("{}_{}", source_filename, s));
        let mut optional_disk_file = match File::open(&path) {
            Err(why) => {
                println!("Cannot open {}: {}", path.display(), why.desc);
                if amount_missing > 0 {
                    panic!("Too many missing disks");
                } else {
                    missing_index = Some(s);
                }
                amount_missing += 1;
                None
            }
            Ok(file) => Some(file)
        };

        match optional_disk_file.as_mut() {
            Some(disk_file) => {
                let read_file_length: u64 = match disk_file.read_le_u64() {
                    Ok(result) => result,
                    Err(why) => panic!("Unable to read the file length from disk {}: {}", s, why.desc),
                };

                match file_length {
                    Some(length) => {
                        if length != read_file_length {
                            panic!("Inconsistent file lengths: {} and {}", length, read_file_length);
                        }
                    }
                    _ => file_length = Some(read_file_length)
                };
            }
            _ => ()
        }

        optional_disk_files.push(optional_disk_file);
    }

    return (optional_disk_files, missing_index, file_length.unwrap() as usize);
}

fn write_chunks_to_file(restored_file: &mut File, chunks: &[Chunk], parity_chunk: usize, mut file_length: usize) -> usize {
    for chunk_index in range(0, chunks.len()) {
        if chunk_index == parity_chunk {
            continue;
        }

        let chunk = chunks[chunk_index];
        let to_write = min(file_length, chunk.len());
        match restored_file.write(&chunk[..to_write]) {
            Ok(_) => file_length -= to_write,
            Err(why) => panic!("Unable to write to the restored file: {}", why.desc)
        }

        if file_length == 0 {
            return 0;
        }
    }

    return file_length;
}

fn read_chunks_from_disks(chunks: &mut [Chunk], disks: &mut [Option<File>]) {
    for i in range(0, disks.len()) {
        let chunk = &mut chunks[i];

        match disks[i].as_mut() {
            None => (),
            Some(disk) => {
                match disk.read(chunk.as_mut_slice()) {
                    Ok(n) => {
                        if n != CHUNK_SIZE {
                            panic!("Read unexpected amount of bytes {} for disk {}", n, i);
                        }
                    },
                    Err(why) => panic!("Could not read from disk {}: {}", i, why.desc),
                }
            }
        };
    }
}

fn reconstruct_chunk(chunks: &mut [Chunk], missing_index: usize) {
    for byte_index in range(0, chunks[missing_index].len()) {
        chunks[missing_index][byte_index] = 0;

        for chunk_index in range(0, chunks.len()) {

            if chunk_index == missing_index {
                continue;
            }

            chunks[missing_index][byte_index] ^= chunks[chunk_index][byte_index];
        }
    }
}

pub fn merge(source_filename: &str, disks: usize) {
    let restored_path = {
        let mut path = Path::new(source_filename);
        let restored_filename = format!("res__{}", &path.filename_str().unwrap());
        path.set_filename(restored_filename);
        path
    };

    let mut restored_file = match File::create(&restored_path) {
        Err(why) => panic!("Unable to open the file for restoration at {}: {}", restored_path.display(), why.desc),
        Ok(file) => file
    };
    println!("Restoring the file to {}", restored_path.display());

    let (mut disk_files, missing_disk, mut file_length) = open_disks(source_filename, disks);
    
    let mut chunks: Vec<Chunk> = Vec::with_capacity(disks as usize);

    for _ in range(0, disks) {
        chunks.push([0; CHUNK_SIZE]);
    };
    
    let mut stripe_number = 0;

    while file_length > 0 {
        let parity_disk = stripe_number % disks;

        read_chunks_from_disks(chunks.as_mut_slice(), disk_files.as_mut_slice());

        match missing_disk {
            Some(missing_disk) if parity_disk != missing_disk => {
                reconstruct_chunk(chunks.as_mut_slice(), missing_disk);
            },
            _ => ()
        }

        file_length = write_chunks_to_file(&mut restored_file, chunks.as_slice(), parity_disk, file_length);
        stripe_number += 1;
    }
}