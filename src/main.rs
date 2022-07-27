use std::fs;

use hound::{SampleFormat, WavReader, WavSpec, WavWriter};

mod decode;
mod encode;

fn main() {
    write();
    read();
}

fn write() {
    let mut writer = WavWriter::create(
        "out.wav",
        WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        },
    )
    .unwrap();

    let encoder = encode::BinEncoder::new(&fs::read("./mango.jpg").unwrap());
    encoder.for_each(|x| writer.write_sample(x).unwrap());
}

fn read() {
    let mut decoder = decode::BinDecoder::new();
    let mut reader = WavReader::open("./out.wav").unwrap();
    reader
        .samples::<f32>()
        .map(|x| x.unwrap())
        .for_each(|x| decoder.add(x));

    fs::write("out.jpg", decoder.done()).unwrap();
}
