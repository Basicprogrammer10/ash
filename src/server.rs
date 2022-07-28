use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::{mpsc, Arc};
use std::thread;

use clap::ArgMatches;
use cpal::{
    self,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use crossbeam::channel;

use crate::coding::BinEncoder;

pub fn start(m: &ArgMatches) {
    let start_cmd = match m.get_one::<String>("RUN_CMD") {
        Some(i) => i,
        None => "bash",
    };
    dbg!(start_cmd);

    // Make encoder
    let (tx, rx) = channel::unbounded::<Vec<u8>>();
    let mut encoder = BinEncoder::new(&[]);

    // Init output stuff
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");
    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range
        .nth(1)
        .expect("no supported config?!")
        .with_sample_rate(cpal::SampleRate(44100));
    let channels = supported_config.channels() as usize;

    // Build stream
    let stream = device
        .build_output_stream(
            &supported_config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                while let Ok(i) = rx.try_recv() {
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
            move |err| println!("[ERR] {}", err),
        )
        .unwrap();

    // Start stream outputting
    stream.play().unwrap();

    // Start program
    let mut parts = start_cmd.split_whitespace();
    let child = Command::new(parts.next().unwrap())
        .args(parts.collect::<Vec<_>>())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to start command");

    let stdout = child.stdout.unwrap();
    let stderr = child.stderr.unwrap();

    let atx = Arc::new(tx);
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

    thread::park();
}
