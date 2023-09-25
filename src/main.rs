use anyhow::{bail, Result};
use itertools::Itertools;
use nom::number::complete::be_u16;
use std::fs::File;
use std::io::prelude::*;

/// Read a 64-bit variable-length int from memory at p[0]
/// Return slice after bytes read and resulting varint (big endian)
fn get_varint(p: &[u8]) -> (&[u8], u64) {
    let mut res = 0;
    let mut i = 0;
    let mut done = false;
    while !done && i < 8 {
        done = (p[i] & 0x80) == 0;
        res <<= 7;
        res |= (0x7F & p[i]) as u64;
        i += 1;
    }
    if !done && i == 8 {
        res <<= 8;
        res |= p[i] as u64;
        i += 1;
    }
    (&p[i..], res)
}

fn main() -> Result<()> {
    // Parse arguments
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        0 | 1 => bail!("Missing <database path> and <command>"),
        2 => bail!("Missing <command>"),
        _ => {}
    }

    // Parse command and act accordingly
    let command = &args[2];
    match command.as_str() {
        ".dbinfo" => {
            let mut file = File::open(&args[1])?;
            let mut header = [0; 100];
            file.read_exact(&mut header)?;

            // The page size is stored at the 16th byte offset, using 2 bytes in
            // big-endian order
            let page_size = u16::from_be_bytes([header[16], header[17]]);
            println!("database page size: {}", page_size);

            // After the 100-byte header (1st page only), is B-tree header
            file.read_exact(&mut header)?;
            let num_tables = u16::from_be_bytes([header[3], header[4]]);

            println!("number of tables: {}", num_tables);
        }
        ".tables" => {
            let mut file = File::open(&args[1])?;
            let mut page = [0; 4096];
            file.read_exact(&mut page)?;

            let num_tables = u16::from_be_bytes([page[103], page[104]]);
            let mut cell_ptr = &page[108..]; // cell pointer array, after B-tree header
            // println!("number of tables: {}", num_tables);

            let mut tables = vec![];
            for _ in 0..num_tables {
                let (next_ptr, addr) = be_u16::<_, ()>(cell_ptr)?;

                // Table B-Tree Leaf Cell
                let table = &page[addr as usize..];
                let (table, nbytes) = get_varint(table); // payload bytes
                let (table, _) = get_varint(table);     // row id

                // A record in the b-tree leaf page
                let record = &table[..nbytes as usize];
                let (header, nbytes) = get_varint(record);
                let body = &record[nbytes as usize..];

                let (header, record_type) = get_varint(header);
                let (_header, serial_type) = get_varint(header);

                let start = ((record_type - 13) >> 1) as usize;
                let end = start + ((serial_type - 13) >> 1) as usize;
                let name = String::from_utf8(body[start..end].into()).unwrap();

                if !name.starts_with("sqlite_") {
                    tables.push(name);
                }

                cell_ptr = next_ptr;
            }

            println!("{}", tables.into_iter().join(" "));
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}
