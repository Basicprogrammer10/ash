use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;

use bitvec::prelude::BitVec;
use clap::ArgMatches;
use cpal::{
    self,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use crossbeam::channel;

use crate::coding::{BinDecoder, BinEncoder};

pub fn start(m: &ArgMatches) {
    let start_cmd = match m.get_one::<String>("RUN_CMD") {
        Some(i) => i,
        None => "bash",
    };
    dbg!(start_cmd);

    // Make encoder
    let (out_tx, out_rx) = channel::unbounded::<Vec<u8>>();
    let (in_tx, in_rx) = channel::unbounded::<Vec<u8>>();
    let mut encoder = BinEncoder::new(&[]);
    let mut decoder = BinDecoder::new();
    let mut decoder_index = 0;

    // Init output stuff
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No output device available");
    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("Error while querying configs");
    let supported_config = supported_configs_range
        .nth(1)
        .expect("No supported config?!")
        .with_sample_rate(cpal::SampleRate(44100));
    let channels = supported_config.channels() as usize;

    // Init input stream
    let input_device = host
        .default_input_device()
        .expect("No input device available");
    let input_config = input_device
        .default_input_config()
        .expect("Error getting config");

    // Build streams
    let error_callback = |err| println!("[ERR] {}", err);
    let stream = device
        .build_output_stream(
            &supported_config.into(),
            move |data: &mut [f32], _| {
                while let Ok(i) = out_rx.try_recv() {
                    encoder.add_data(&i);
                }

                let mut last = 0.0;
                for (i, x) in data.iter_mut().enumerate() {
                    if i % channels == 0 {
                        last = encoder.next().unwrap_or(0.);
                    }

                    *x = last;
                }
            },
            error_callback,
        )
        .unwrap();

    let input_stream = input_device
        .build_input_stream(
            &input_config.into(),
            move |data: &[f32], _| {
                data.iter().for_each(|x| decoder.add(*x));

                let new_bits = (decoder.data.len() - decoder_index) / 8;
                if new_bits > 0 {
                    in_tx
                        .send(
                            decoder
                                .data
                                .iter()
                                .skip(decoder_index)
                                .take(new_bits * 8)
                                .collect::<BitVec<_>>()
                                .into_vec(),
                        )
                        .unwrap();
                    decoder_index = new_bits * 8;
                }
            },
            error_callback,
        )
        .unwrap();

    // Start stream outputting
    stream.play().unwrap();
    input_stream.play().unwrap();

    // Start program
    let mut parts = start_cmd.split_whitespace();
    let child = Command::new(parts.next().unwrap())
        .args(parts.collect::<Vec<_>>())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to start command");

    let stdout = child.stdout.unwrap();
    let stderr = child.stderr.unwrap();
    let mut stdin = child.stdin.unwrap();

    let atx = Arc::new(out_tx);
    let tx = atx.clone();
    thread::spawn(move || {
        for i in stdout.bytes().map(|x| x.unwrap()) {
            tx.send(vec![i]).unwrap();
        }
    });

    let tx = atx.clone();
    thread::spawn(move || {
        for i in stderr.bytes().map(|x| x.unwrap()) {
            tx.send(vec![i]).unwrap();
        }
    });

    for i in in_rx.iter() {
        stdin.write_all(&i).unwrap();
    }

    thread::park();
}
