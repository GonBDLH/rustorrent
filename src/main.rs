use std::{
    fs::File,
    io::{BufReader, Read},
};

mod metainfo_decoder;

use metainfo_decoder::TorrentDecoder;

fn main() {
    let file = File::open("torrents/sample.torrent").unwrap();
    let mut read_buffer = BufReader::new(file);

    let mut buffer = Vec::new();
    let _bytes_read = read_buffer.read_to_end(&mut buffer).unwrap();

    let mut decoder = TorrentDecoder::new(&buffer);

    let metainfo = decoder.decode_metafile();

    println!("{:?}", metainfo);
}
