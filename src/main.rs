use std::fs;

use bitvec::{order::Lsb0, vec::BitVec, view::BitView};
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};

mod encode;

fn main() {
    // write();
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
    let mut reader = WavReader::open("./out.wav").unwrap();
    let samples = reader
        .samples::<f32>()
        .map(|x| x.unwrap())
        .collect::<Vec<_>>();

    let mut i = 1;
    let mut start = 0;
    let mut data = BitVec::<u8, Lsb0>::new();
    while i < samples.len() {
        let value = samples[i];
        if value < 0. && samples[i - 1] >= 0. {
            data.push(i - start > 50);
            start = i;
        }

        i += 1;
    }

    fs::write("out.jpg", data.into_vec()).unwrap();
}
