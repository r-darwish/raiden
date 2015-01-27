use std::io::{File, IoError, IoErrorKind};
use super::{Chunk, create_chunks_vec};

fn create_disks(source_filename: &str, disks: usize, source_file_length: u64)
        -> Result<Vec<File>, IoError> {
    let mut disk_files: Vec<File> = Vec::with_capacity(disks);

    for s in range(0, disks) {
        let path = Path::new(format!("{}_{}", source_filename, s));
        let mut disk_file = try!(File::create(&path));
        try!(disk_file.write_le_u64(source_file_length));
        disk_files.push(disk_file);
    }

    Ok(disk_files)
}

fn write_to_disks(disk_files: &mut [File], chunks: &[Chunk])
        -> Result<(), IoError> {
    for (disk, chunk) in disk_files.iter_mut().zip(chunks.iter()) {
        try!(disk.write(chunk));
    }

    Ok(())
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

fn read_to_chunk(source_file: &mut File, chunk: &mut Chunk) -> Result<bool, IoError> {
    match source_file.read(chunk) {
        Ok(n) => {
            if n < chunk.len() {
                // Partial read. Zero the rest of the chunk
                for byte_i in range(n, chunk.len()) {
                    chunk[byte_i] = 0;
                }
            }
        },
        Err(why) => {
            if why.kind == IoErrorKind::EndOfFile {
                return Ok(true)
            } else {
                return Err(why);
            }
        }
    };

    Ok(false)
}

fn read_to_chunks(source_file: &mut File, chunks: &mut [Chunk], parity_chunk: usize)
        -> Result<bool, IoError> {
    let mut eof_reached = false;

    for chunk_i in range(0, chunks.len()) {
        if chunk_i == parity_chunk {
            continue;
        }

        if !eof_reached {
            eof_reached = try!(read_to_chunk(source_file, &mut chunks[chunk_i]));
        }

        // We shouldn't write this as an else statement because eof_reached might be set to
        // true in the block above
        if eof_reached {
            for bytes_i in range(0, chunks[chunk_i].len()) {
                chunks[chunk_i][bytes_i] = 0;
            }
        }
    }

    Ok(eof_reached)
}

pub fn split(source_filename: &str, disks: usize) -> Result<(), IoError> {
    let mut source_file = try!(File::open(&Path::new(source_filename)));
    let file_length = try!(source_file.stat()).size;
    let mut disk_files = try!(create_disks(source_filename, disks, file_length));
    let mut chunks = create_chunks_vec(disks);
    let mut eof_reached = false;
    let mut parity_disk_iterator = range(0, disks).cycle();

    while !eof_reached {
        let parity_disk = parity_disk_iterator.next().unwrap() as usize;

        eof_reached = try!(read_to_chunks(&mut source_file, chunks.as_mut_slice(), parity_disk));
        calculate_parity(chunks.as_mut_slice(), parity_disk);
        try!(write_to_disks(disk_files.as_mut_slice(), chunks.as_mut_slice()));
    }

    return Ok(())
}