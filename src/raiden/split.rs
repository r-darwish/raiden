use std::io::File;
use std::io::IoErrorKind;
use super::types::{Chunk,CHUNK_SIZE};

fn create_disks(source_filename: &str, disks: usize, source_file_length: u64) -> Vec<File> {
    let mut disk_files: Vec<File> = Vec::with_capacity(disks);

    for s in range(0, disks) {
        let path = Path::new(format!("{}_{}", source_filename, s));
        let mut disk_file = match File::create(&path) {
            Err(why) => panic!("Cannot create {}: {}", path.display(), why.desc),
            Ok(file) => file
        };

        match disk_file.write_le_u64(source_file_length) {
            Err(why) => panic!("Unable to write the source file length to disk {}: {}", s, why.desc),
            _ => ()
        }

        disk_files.push(disk_file);
    }

    return disk_files;
}

fn write_to_disks(disk_files: &mut [File], chunks: &[Chunk]) {
    for i in range(0, disk_files.len()) {
        match disk_files[i].write(&chunks[i]) {
            Err(why) => panic!("Cannot write to disk {}: {}", i, why.desc),
            _ => ()
        }
    }
}

fn calculate_parity(chunks: &mut [Chunk], parity_chunk: usize) {
    for byte_i in range(0, chunks[parity_chunk].len()) {
        chunks[parity_chunk][byte_i] = 0;

        for chunk_i in range(0, chunks.len()) {
            if chunk_i == parity_chunk {
                continue;
            }

            chunks[parity_chunk][byte_i] ^= chunks[chunk_i][byte_i];
        }
    }
}

fn read_to_chunks(source_file: &mut File, chunks: &mut [Chunk], parity_chunk: usize) -> bool {
    let mut eof_reached = false;

    for chunk_i in range(0, chunks.len()) {
        if chunk_i == parity_chunk {
            continue;
        }

        if !eof_reached {
            match source_file.read(&mut chunks[chunk_i]) {
                Ok(n) => {
                    if n < chunks[chunk_i].len() {
                        // Partial read. Zero the rest of the chunk
                        for byte_i in range(n, chunks[chunk_i].len()) {
                            chunks[chunk_i][byte_i] = 0;
                        }
                    }
                },
                Err(why) => {
                    if why.kind == IoErrorKind::EndOfFile {
                        eof_reached = true;
                    } else {
                        panic!("Error reading: {}", why.desc);
                    }
                }
            };
        }

        // We shouldn't write this as an else statement because eof_reached might be set to
        // true in the block above
        if eof_reached {
            for bytes_i in range(0, chunks[chunk_i].len()) {
                chunks[chunk_i][bytes_i] = 0;
            }
        }
    }

    return eof_reached;
}

pub fn split(source_filename: &str, disks: usize) {
    let mut source_file = match File::open(&Path::new(source_filename)) {
        Err(why) => panic!("Unable to open the source file {}", why.desc),
        Ok(file) => file
    };

    let file_length = match source_file.stat() {
        Ok(stat) => stat.size,
        Err(why) => panic!("Cannot stat {}: {}", source_filename, why.desc),
    };

    let mut disk_files = create_disks(source_filename, disks, file_length);

    let mut chunks: Vec<Chunk> = Vec::with_capacity(disks);
    for _ in range(0, disks) {
        chunks.push([0; CHUNK_SIZE]);
    }
    
    let mut stripe_number = 0;
    let mut eof_reached = false;

    while !eof_reached {
        let parity_disk = stripe_number % disks;

        eof_reached = read_to_chunks(&mut source_file, chunks.as_mut_slice(), parity_disk);
        calculate_parity(chunks.as_mut_slice(), parity_disk);
        write_to_disks(disk_files.as_mut_slice(), chunks.as_mut_slice());
        stripe_number += 1;
    }
}