use std::io::{Read, Seek, SeekFrom};

use memchr;

use error::*;

pub(crate) fn preamble_skipcount<R: Read>(reader: &mut R, n_preamble_rows: usize)
    -> Result<usize>
{
    if n_preamble_rows == 0 {
        return Ok(0);
    }
    let mut skipcount = 0;
    loop {
        let cap = 1 << 12;
        let mut buffer = vec![0; cap];
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
            skipcount += crlf_pos;
            break;
        } else {
            skipcount += cap.min(n_read);
        }
    }
    Ok(skipcount)
}

pub(crate) fn snip_preamble<R: Read + Seek>(mut reader: R, n_preamble_rows: usize) -> Result<()> {
    let seek_point = preamble_skipcount(&mut reader, n_preamble_rows)?;
    reader.seek(SeekFrom::Start(seek_point as u64))?;
    Ok(())
}
