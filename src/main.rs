use clap::{builder::NonEmptyStringValueParser, Arg, Command};

mod client;
mod coding;
mod server;

fn main() {
    let m = Command::new("Audio Term")
        .author("Connor Slade")
        .version("0.0.0")
        .subcommand_required(true)
        .subcommands([
            Command::new("client").about("Start the audio term client"),
            Command::new("server")
                .about("Start the audio term server")
                .arg(
                    Arg::new("cmd")
                        .id("RUN_CMD")
                        .takes_value(true)
                        .value_parser(NonEmptyStringValueParser::new())
                        .value_name("RUN COMMAND")
                        .help("The command to open a shell"),
                ),
        ])
        .get_matches();

    match m.subcommand() {
        Some(("client", m)) => client::start(m),
        Some(("server", m)) => server::start(m),
        _ => panic!("Invalid Subcommand"),
    };
}

// fn write() {
//     let mut writer = WavWriter::create(
//         "out.wav",
//         WavSpec {
//             channels: 1,
//             sample_rate: 44100,
//             bits_per_sample: 32,
//             sample_format: SampleFormat::Float,
//         },
//     )
//     .unwrap();
//
//     let encoder = coding::BinEncoder::new(&fs::read("./mango.jpg").unwrap());
//     encoder.for_each(|x| writer.write_sample(x).unwrap());
// }
//
// fn read() {
//     let mut decoder = coding::BinDecoder::new();
//     let mut reader = WavReader::open("./out.wav").unwrap();
//     reader
//         .samples::<f32>()
//         .map(|x| x.unwrap())
//         .for_each(|x| decoder.add(x));
//
//     fs::write("out.jpg", decoder.done()).unwrap();
// }
