use std::old_io::{File, Reader, IoError, IoErrorKind};
use std::cmp::min;
use super::{Chunk, create_chunks_vec};

fn length_inconsistency_error(a: usize, b: usize) -> IoError {
    IoError {
        kind: IoErrorKind::OtherIoError,
        desc: "File length inconsistency",
        detail: Some(format!("{} != {}", a, b))}
}

struct DiskArray {
    disks: Vec<Option<File>>,
    missing_disk: Option<usize>,
    file_length: usize
}

fn open_disk(source_filename: &str, disk: usize) -> Option<(File, usize)> {
    let path = Path::new(format!("{}_{}", source_filename, disk));
    let mut disk = match File::open(&path) {
        Err(why) => {
            println!("{}", why);
            return None;
        }
        Ok(file) => file
    };

    let file_length = match disk.read_le_u64() {
        Err(why) => {
            println!("{}", why);
            return None;
        }
        Ok(n) => n
    };

    Some((disk, file_length as usize))
}

fn open_disks(source_filename: &str, disks: usize) -> Result<DiskArray, IoError> {
    let mut optional_disk_files: Vec<Option<File>> = Vec::with_capacity(disks as usize);
    let mut missing_index = None;
    let mut saved_file_length: Option<usize> = None;

    for s in range(0, disks) {
        match open_disk(source_filename, s) {
            Some((disk, file_length)) => {
                match saved_file_length {
                    Some(saved_file_length) => {
                        if saved_file_length != file_length {
                            return Err(length_inconsistency_error(file_length, saved_file_length));
                        }
                    },
                    _ => saved_file_length = Some(file_length),
                }
                optional_disk_files.push(Some(disk));
            }
            _ => {
                match missing_index {
                    Some(_) => return Err(IoError {
                        kind: IoErrorKind::OtherIoError, desc: "Too Many missing disks", detail: None }),
                    _ => missing_index = Some(s),
                }
                optional_disk_files.push(None);
            }
        }
    }

    match saved_file_length {
        Some(saved_file_length) =>
            Ok(DiskArray {
                disks: optional_disk_files,
                missing_disk: missing_index,
                file_length: saved_file_length}),
        _ =>
            Err(IoError {
                kind: IoErrorKind::OtherIoError, desc: "Could not load any disks", detail: None })
    }
}

fn write_chunks_to_file(restored_file: &mut File, chunks: &[Chunk], parity_chunk: usize, mut bytes_remaining: usize)
        -> Result<usize, IoError> {
    for chunk_index in range(0, chunks.len()) {
        if chunk_index == parity_chunk {
            continue;
        }

        let chunk = chunks[chunk_index];
        let to_write = min(bytes_remaining, chunk.len());
        try!(restored_file.write_all(&chunk[..to_write]));
        bytes_remaining -= to_write;

        if bytes_remaining == 0 {
            return Ok(0);
        }
    }

    return Ok(bytes_remaining);
}

fn read_chunks_from_disks(chunks: &mut [Chunk], disks: &mut [Option<File>]) -> Result<(), IoError> {
    for i in range(0, disks.len()) {
        let chunk = &mut chunks[i];

        match disks[i].as_mut() {
            None => (),
            Some(disk) => {
                let bytes_read = try!(disk.read(chunk.as_mut_slice()));
                if bytes_read != chunk.len() {
                    return Err(IoError {
                        kind: IoErrorKind::OtherIoError,
                        desc: "Read a partial chunk",
                        detail: Some(format!("disk: {}, bytes read: {}", i, bytes_read))});
                }
            }
        };
    }

    Ok(())
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

fn get_restored_path(source_filename: &str) -> Path {
    let mut path = Path::new(source_filename);
    let restored_filename = format!("res__{}", &path.filename_str().unwrap());
    path.set_filename(restored_filename);
    path
}

pub fn merge(source_filename: &str, disks: usize) -> Result<(), IoError> {
    let mut disk_array = try!(open_disks(source_filename, disks));
    let mut restored_file = try!(File::create(&get_restored_path(source_filename)));
    let mut remaining_bytes = disk_array.file_length;
    let mut chunks = create_chunks_vec(disks);
    let mut parity_disk_iterator = range(0, disks).cycle();

    while remaining_bytes > 0 {
        let parity_disk = parity_disk_iterator.next().unwrap() as usize;

        try!(read_chunks_from_disks(chunks.as_mut_slice(), disk_array.disks.as_mut_slice()));

        match disk_array.missing_disk {
            Some(missing_disk) if parity_disk != missing_disk => {
                reconstruct_chunk(chunks.as_mut_slice(), missing_disk);
            },
            _ => ()
        }

        remaining_bytes = try!(
            write_chunks_to_file(&mut restored_file, chunks.as_slice(), parity_disk, remaining_bytes));
    }

    Ok(())
}