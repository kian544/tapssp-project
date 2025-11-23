use rodio::{Decoder, OutputStream, Sink, source::Source};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

pub struct Music {
    // Keep stream alive for the life of the program
    _stream: OutputStream,
    sink: Sink,
}

impl Music {
    /// Start looping background music from a file path.
    pub fn start_loop<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        // Make path absolute relative to project root if it's relative.
        let abs_path = make_abs(path.as_ref());
        let file = File::open(&abs_path)?;
        
        // Use MP3 hint decoder (now supported via feature flag)
        let source = Decoder::new_mp3(BufReader::new(file))?.repeat_infinite();

        sink.append(source);
        sink.play();

        Ok(Self { _stream, sink })
    }

    #[allow(dead_code)]
    pub fn stop(self) {
        self.sink.stop();
    }
}

fn make_abs(p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(p)
    }
}
