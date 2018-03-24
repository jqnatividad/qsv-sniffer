use std::io::{Read, Seek, SeekFrom};

use memchr;

use error::*;

pub(crate) fn snip_preamble<R: Read + Seek>(mut reader: R, n_preamble_rows: usize) -> Result<()> {
    let mut seek_point = 0;
    loop {
        let cap = 1 << 12;
        let mut buffer = Vec::with_capacity(cap);
        unsafe { buffer.set_len(cap); }
        let n_read = reader.read(&mut buffer)?;
        let mut crlf_pos = 0;
        let mut found = true;
        for _ in 0..n_preamble_rows {
            match memchr::memchr(b'\n', &buffer[crlf_pos..]) {
                Some(pos) => {
                    crlf_pos += pos + 1;
                },
                None => {
                    found = false;
                    break;
                }
            }
        }
        if found {
            seek_point += crlf_pos;
            break;
        } else {
            seek_point += cap.min(n_read);
        }
    }
    reader.seek(SeekFrom::Start(seek_point as u64))?;
    Ok(())
}
