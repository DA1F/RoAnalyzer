// This code demonstrates a synchronized video and audio recording system
// using FFmpeg in Rust. It captures raw video frames and audio samples,
// encodes them, and muxes them into a single MP4 file with proper synchronization.

use crate::proto::emulator_controller_client::EmulatorControllerClient;
use crate::proto::{AudioPacket, DisplayConfigurations, Image};
use anyhow::Result;
use ffmpeg_next as ffmpeg;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tonic::transport::Channel;
use tonic::{Status, Streaming};

// --- 1. Define Input Structures ---

#[derive(Debug, Clone)]
pub struct VideoRecoarder {
    inner: EmulatorControllerClient<Channel>,
    display_index: u32,
    /// Recording duration in seconds (0 for indefinite)
    duration_secs: u64,
    output_path: PathBuf,
    include_audio: bool,
    /// Frame rate for video capture (frames per second)
    fps: u32,
    /// Width of the captured video (0 for native resolution)
    width: u32,
    /// Height of the captured video (0 for native resolution)
    height: u32,
    /// Audio sample rate (Hz), only used if include_audio is true (Default 44100)
    audio_sample_rate: u64,
}

impl VideoRecoarder {
    pub fn new(inner: EmulatorControllerClient<Channel>) -> Self {
        Self {
            inner: inner,
            display_index: 0,
            duration_secs: 0,
            output_path: PathBuf::from("output.mp4"),
            include_audio: false,
            fps: 30,
            width: 0,
            height: 0,
            audio_sample_rate: 44100,
        }
    }

    async fn get_display_configurations(&mut self) -> Result<DisplayConfigurations, Status> {
        let req = tonic::Request::new(());
        let resp = self.inner.get_display_configurations(req).await?;
        Ok(resp.into_inner())
    }

    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }
    pub fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }
    pub fn display_index(mut self, display_index: u32) -> Self {
        self.display_index = display_index;
        self
    }
    pub fn audio_sample_rate(mut self, sample_rate: u64) -> Self {
        self.audio_sample_rate = sample_rate;
        self
    }
    pub fn fps(mut self, fps: u32) -> Self {
        self.fps = fps;
        self
    }
    pub fn duration_secs(mut self, duration_secs: u64) -> Self {
        self.duration_secs = duration_secs;
        self
    }
    pub fn output_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.output_path = path.as_ref().to_path_buf();
        self
    }
    pub fn include_audio(mut self, include: bool) -> Self {
        self.include_audio = include;
        self
    }

    pub async fn start(&mut self) {
        if self.width == 0 || self.height == 0 {
            let display_config = self.get_display_configurations().await.unwrap();
            let display = display_config
                .displays
                .get(self.display_index as usize)
                .unwrap();
            self.width = display.width;
            self.height = display.height;
        }
        println!(
            "\x1bStarting recording display {} with resolution {}x{}\x1b[0m",
            self.display_index, self.width, self.height
        );
    }
    pub fn stop(&self) {
        // Implementation to stop recording goes here.
        println!("\x1b[1m--------------------\nStopping recording...\x1b[0m");
    }
}

/// Represents a raw RGB video frame received from the emulator stream.
#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub timestamp_ms: u64,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGB888 pixel data (W * H * 3 bytes)
}

///Represents raw audio samples received from the emulator stream.
#[derive(Debug, Clone)]
pub struct InputAudioFrame {
    pub timestamp_ms: u64,
    pub sample_rate: u32,
    pub channels: u8,
    pub data: Vec<u8>, // Raw 16-bit PCM audio samples (stereo/mono)
}

// /// Represents an encoded packet ready to be written to the file.
// /// Used to interleave video and audio packets correctly.
pub struct ReadyPacket {
    pub pts: i64,            // Presentation Timestamp (scaled to output time base)
    pub stream_index: usize, // 0 for video, 1 for audio
    pub data: Vec<u8>,       // Raw encoded packet data
    pub stream_time_base: ffmpeg::Rational, // Time base of the output stream
}

// // --- 3. Encoder Functions (CPU-Bound, run in dedicated thread) ---

// // Uses a dedicated struct to hold the complex FFmpeg encoder state.
struct VideoEncoderState {
    encoder: ffmpeg::encoder::Video,
    scaler: ffmpeg::software::scaling::Context,
    video_stream_index: usize,
    stream_time_base: ffmpeg::Rational,
}

struct AudioEncoderState {
    encoder: ffmpeg::encoder::Audio,
    resampler: ffmpeg::software::resampling::Context,
    audio_stream_index: usize,
    stream_time_base: ffmpeg::Rational,
    frame_count: i64,
    // Note: In a real app, you need a buffer to accumulate partial samples
    // that don't fill the encoder's required frame size.
}

// /// The video encoder consumer. Takes raw RGB frames and outputs compressed packets.
// fn video_encoder_consumer(
//     mut rx: mpsc::Receiver<Image>,
//     tx_muxer: mpsc::Sender<ReadyPacket>,
//     mut state: VideoEncoderState,
// ) -> Result<()> {
//     // We use a counter to ensure we track the PTS manually
//     let mut current_pts = 0;
//     let time_base = state.encoder.time_base();

//     while let Some(frame) = rx.blocking_recv() {
//         // 1. Create Input Frame (RGB24)
//         let mut rgb_frame = ffmpeg::util::frame::video::Video::new(
//             ffmpeg::format::Pixel::RGB24,
//             frame.width,
//             frame.height,
//         );
//         rgb_frame.data_mut(0).copy_from_slice(&frame.data);

//         // 2. Scale and Convert to YUV420P
//         let mut yuv_frame = ffmpeg::util::frame::video::Video::empty();
//         state.scaler.run(&rgb_frame, &mut yuv_frame)?;

//         // Set Presentation Timestamp (PTS)
//         // Use frame count for simple sequencing, or convert frame.timestamp_ms
//         yuv_frame.set_pts(Some(current_pts));
//         current_pts += time_base.den() as i64 / time_base.num() as i64 / 30; // approx 33 ms per frame at 30fps

//         // 3. Encode the frame
//         state.encoder.send_frame(&yuv_frame)?;

//         // 4. Send encoded packets to the muxer
//         let mut encoded_packet = ffmpeg::codec::packet::Packet::empty();
//         while state.encoder.receive_packet(&mut encoded_packet).is_ok() {
//             let ready_packet = ReadyPacket {
//                 pts: encoded_packet.pts().unwrap_or(0),
//                 stream_index: state.video_stream_index,
//                 data: encoded_packet
//                     .data()
//                     .map(|d| d.to_vec())
//                     .unwrap_or_default(),
//                 stream_time_base: state.stream_time_base,
//             };
//             if tx_muxer.blocking_send(ready_packet).is_err() {
//                 println!("Muxer channel closed, stopping video encoder.");
//                 return Ok(());
//             }
//         }
//     }

//     // 5. Flush the encoder
//     state.encoder.send_eof()?;
//     let mut encoded_packet = ffmpeg::codec::packet::Packet::empty();
//     while state.encoder.receive_packet(&mut encoded_packet).is_ok() {
//         // Send remaining packets (flush)
//         // ... (similar to step 4) ...
//     }

//     Ok(())
// }

// /// The audio encoder consumer. Takes raw PCM samples and outputs compressed packets.
// fn audio_encoder_consumer(
//     mut rx: mpsc::Receiver<InputAudioFrame>,
//     tx_muxer: mpsc::Sender<ReadyPacket>,
//     mut state: AudioEncoderState,
// ) -> Result<()> {
//     // The PTS timebase for audio is 1 / sample_rate
//     let mut current_pts = 0;

//     while let Some(audio_frame) = rx.blocking_recv() {
//         // 1. Create Input Frame (raw 16-bit PCM)
//         let mut raw_pcm_frame = ffmpeg::util::frame::audio::Audio::new(
//             ffmpeg::format::sample::Type::I16(ffmpeg::format::sample::IsPlanar::Packed),
//             audio_frame.data.len() as u32 / 4, // Calculate samples per channel (16-bit, stereo)
//             ffmpeg::channel_layout::CH_LAYOUT_STEREO,
//         );
//         raw_pcm_frame.data_mut(0).copy_from_slice(&audio_frame.data);

//         // 2. Resample (if necessary)
//         // In this example, we skip the resampler for simplicity, assuming I16 matches the encoder's needs.

//         // 3. Encode the frame
//         let frame_size = state.encoder.frame_size();
//         // NOTE: Real AAC encoding requires accumulating samples until they fill frame_size

//         // Simulate encoding by sending the raw frame, though AAC encoder is more complex
//         raw_pcm_frame.set_pts(Some(current_pts));
//         current_pts += raw_pcm_frame.samples() as i64;

//         state.encoder.send_frame(&raw_pcm_frame)?;

//         // 4. Send encoded packets to the muxer
//         let mut encoded_packet = ffmpeg::codec::packet::Packet::empty();
//         while state.encoder.receive_packet(&mut encoded_packet).is_ok() {
//             let ready_packet = ReadyPacket {
//                 pts: encoded_packet.pts().unwrap_or(0),
//                 stream_index: state.audio_stream_index,
//                 data: encoded_packet
//                     .data()
//                     .map(|d| d.to_vec())
//                     .unwrap_or_default(),
//                 stream_time_base: state.stream_time_base,
//             };
//             if tx_muxer.blocking_send(ready_packet).is_err() {
//                 println!("Muxer channel closed, stopping audio encoder.");
//                 return Ok(());
//             }
//         }
//     }

//     // 5. Flush the encoder
//     // ... (similar to video flush) ...
//     Ok(())
// }

// // --- 4. Muxer Consumer Function (The Synchronizer) ---

/// The final consumer. Receives packets from both video/audio encoders and interleaves them.
// fn muxer_consumer(mut rx_muxer: mpsc::Receiver<ReadyPacket>, output_path: &Path) -> Result<()> {
//     // 1. Setup Output Muxer
//     let mut output_context =
//         ffmpeg::format::output(&output_path).context("Failed to open output file context")?;

//     // The output context streams must be manually initialized to match
//     // the stream indices used by the encoders (0 for video, 1 for audio)
//     // In a real setup, this is tricky. We rely on the initial setup in main.

//     output_context
//         .write_header()
//         .context("Failed to write MP4 header")?;

//     println!("MP4 Muxer started, waiting for packets...");

//     // 2. Main Interleaving Loop
//     // The packets are received out of order, so we collect them and sort them by PTS.
//     let mut packet_buffer: Vec<ReadyPacket> = Vec::new();

//     loop {
//         // Wait for the next packet
//         match rx_muxer.blocking_recv() {
//             Some(packet) => {
//                 packet_buffer.push(packet);

//                 // Sort the buffer by Presentation Timestamp (PTS)
//                 packet_buffer.sort_by_key(|p| p.pts);

//                 // Interleave: Write the packet with the lowest PTS and remove it.
//                 if let Some(p) = packet_buffer.first() {
//                     let mut ffmpeg_packet = ffmpeg::codec::packet::Packet::copy(p.data.as_slice());
//                     ffmpeg_packet.set_stream(p.stream_index);
//                     ffmpeg_packet.set_pts(Some(p.pts));

//                     // Rescale PTS/DTS from the stream time base to the global output context time base.
//                     // This is the most crucial part for synchronization.
//                     // NOTE: A proper implementation requires knowing the input stream's time base
//                     // for rescale_ts, which is managed internally by the encoder.
//                     // We simplify here by assuming the encoder already provided the correct PTS.

//                     output_context
//                         .write_packet(&ffmpeg_packet)
//                         .context(format!(
//                             "Failed to write packet for stream {}",
//                             p.stream_index
//                         ))?;

//                     packet_buffer.remove(0);
//                 }
//             }
//             None => {
//                 // All senders closed (streams finished). Flush remaining buffer.
//                 println!(
//                     "All streams closed. Flushing {} remaining packets.",
//                     packet_buffer.len()
//                 );
//                 packet_buffer.sort_by_key(|p| p.pts);

//                 for p in packet_buffer.into_iter() {
//                     let mut ffmpeg_packet = ffmpeg::codec::packet::Packet::copy(p.data.as_slice());
//                     ffmpeg_packet.set_stream(p.stream_index);
//                     ffmpeg_packet.set_pts(Some(p.pts));
//                     output_context.write_packet(&ffmpeg_packet)?;
//                 }
//                 break;
//             }
//         }
//     }

//     output_context.write_trailer()?;
//     println!("Muxing complete. File saved to: {}", output_path.display());
//     Ok(())
// }

// // --- 5. Main Execution ---

// #[tokio::main]
// async fn main() -> Result<()> {
//     // 1. Configuration
//     let output_file = PathBuf::from("emulator_capture_sync.mp4");

//     // Use a large channel capacity to prevent back-pressure from the CPU-heavy encoder,
//     // sacrificing memory for stream smoothness.
//     const CHANNEL_CAPACITY: usize = 100;

//     // Channels from Producer (gRPC Simulators) to Encoders
//     let (tx_video_in, rx_video_in) = mpsc::channel(CHANNEL_CAPACITY);
//     let (tx_audio_in, rx_audio_in) = mpsc::channel(CHANNEL_CAPACITY);

//     // Channel from Encoders to Muxer (The Synchronization Point)
//     let (tx_muxer, rx_muxer) = mpsc::channel(CHANNEL_CAPACITY);

//     // 2. FFmpeg Global Initialization (must be called once)
//     ffmpeg::init().context("Failed to initialize FFmpeg")?;

//     // 3. Output Muxer Setup (Needs to happen on the main thread to configure encoders)
//     let mut output_context =
//         ffmpeg::format::output(&output_file).context("Failed to open output file context")?;

//     // --- Video Stream Setup ---
//     let video_codec =
//         ffmpeg::encoder::find_by_name("libx264").context("H.264 encoder not found")?;
//     let mut video_stream = output_context.add_stream(video_codec)?;
//     let video_stream_idx = video_stream.index();
//     let video_time_base = ffmpeg::Rational::new(1, 1000); // ms time base

//     let mut video_encoder = {
//         let mut encoder = video_stream.codec().encoder().video()?;
//         encoder.set_width(VIDEO_WIDTH);
//         encoder.set_height(VIDEO_HEIGHT);
//         encoder.set_time_base(video_time_base);
//         encoder.set_format(ffmpeg::format::Pixel::YUV420P);
//         encoder.set_frame_rate((30, 1));
//         encoder.set_bit_rate(4000000); // 4 Mbps
//         encoder.set_parameters([("preset", "ultrafast")])?;
//         encoder.open_as(video_codec)?
//     };

//     let video_scaler = ffmpeg::software::scaling::Context::get(
//         VIDEO_WIDTH,
//         VIDEO_HEIGHT,
//         ffmpeg::format::Pixel::RGB24,
//         VIDEO_WIDTH,
//         VIDEO_HEIGHT,
//         video_encoder.format(),
//         ffmpeg::software::scaling::flag::SWS_BILINEAR,
//     )?;

//     let video_state = VideoEncoderState {
//         encoder: video_encoder,
//         scaler: video_scaler,
//         video_stream_index: video_stream_idx,
//         stream_time_base: video_stream.time_base(),
//     };

//     // --- Audio Stream Setup ---
//     let audio_codec =
//         ffmpeg::encoder::find(ffmpeg::codec::Id::AAC).context("AAC encoder not found")?;
//     let mut audio_stream = output_context.add_stream(audio_codec)?;
//     let audio_stream_idx = audio_stream.index();
//     let audio_time_base = ffmpeg::Rational::new(1, AUDIO_SAMPLE_RATE as i32); // 1/44100 sec

//     let mut audio_encoder = {
//         let mut encoder = audio_stream.codec().encoder().audio()?;
//         encoder.set_time_base(audio_time_base);
//         encoder.set_sample_rate(AUDIO_SAMPLE_RATE);
//         encoder.set_channel_layout(ffmpeg::channel_layout::CH_LAYOUT_STEREO);
//         encoder.set_format(ffmpeg::format::sample::Type::FLT(
//             ffmpeg::format::sample::IsPlanar::Packed,
//         )); // AAC often prefers float
//         encoder.open_as(audio_codec)?
//     };

//     // NOTE: In a real implementation, you would need an audio resampler here
//     // to convert I16 (input) to FLT (encoder format). We skip for simplicity.
//     let audio_resampler = ffmpeg::software::resampling::Context::get(
//         audio_encoder.channel_layout(),
//         audio_encoder.sample_rate(),
//         audio_encoder.format(),
//         ffmpeg::channel_layout::CH_LAYOUT_STEREO,
//         AUDIO_SAMPLE_RATE,
//         ffmpeg::format::sample::Type::I16(ffmpeg::format::sample::IsPlanar::Packed),
//     )?;

//     let audio_state = AudioEncoderState {
//         encoder: audio_encoder,
//         resampler: audio_resampler,
//         audio_stream_index: audio_stream_idx,
//         stream_time_base: audio_stream.time_base(),
//         frame_count: 0,
//     };

//     // 4. Launch Tasks
//     println!("Starting video and audio stream producers...");

//     // Producers (Async, run on Tokio runtime)
//     let producer_video_handle = task::spawn(video_producer(tx_video_in.clone()));
//     let producer_audio_handle = task::spawn(audio_producer(tx_audio_in.clone()));

//     // Consumers (Blocking/CPU-heavy, run in dedicated blocking pool)
//     let encoder_video_handle = task::spawn_blocking(move || {
//         video_encoder_consumer(rx_video_in, tx_muxer.clone(), video_state)
//     });
//     let encoder_audio_handle = task::spawn_blocking(move || {
//         audio_encoder_consumer(rx_audio_in, tx_muxer.clone(), audio_state)
//     });

//     // Muxer (Blocking, runs in dedicated blocking pool)
//     let muxer_handle = task::spawn_blocking(move || muxer_consumer(rx_muxer, &output_file));

//     // 5. Wait for all tasks to complete
//     let _ = tokio::join!(
//         producer_video_handle,
//         producer_audio_handle,
//         encoder_video_handle,
//         encoder_audio_handle,
//         muxer_handle
//     );

//     Ok(())
// }

// //

// pub struct GrpcVideoClient {
//     inner: EmulatorControllerClient<Channel>,
// }

// impl GrpcVideoClient {
//     /// Connect to the gRPC endpoint (e.g., "127.0.0.1:8701").
//     pub async fn connect(endpoint: impl Into<String>) -> Result<Self, Box<dyn std::error::Error>> {
//         let ep = endpoint.into();
//         let channel = Channel::from_shared(ep)?.connect().await?;
//         let inner = EmulatorControllerClient::new(channel);
//         Ok(Self { inner })
//     }

//     pub async fn recoard_video(
//         &mut self,
//         duration_secs: u64,
//         path: &Path,
//         config: RecordingConfig,
//     ) -> Result<(), Box<dyn std::error::Error>> {
//         use chrono::DateTime;
//         let displays_config = self.get_display_configurations().await?;
//         let main_display = displays_config.displays.first().ok_or("No display found")?;
//         let VIDEO_WIDTH = if config.width > 0 {
//             config.width
//         } else {
//             main_display.width
//         };
//         let VIDEO_HEIGHT = if config.height > 0 {
//             config.height
//         } else {
//             main_display.height
//         };
//         let fps = config.fps;
//         let img_format = ImageFormat {
//             format: proto::image_format::ImgFormat::Rgb888 as i32,
//             rotation: None,
//             width: main_display.width,
//             height: main_display.height,
//             display: 0,
//             transport: None,
//             folded_display: None,
//             display_mode: 0,
//         };
//         let mut video_stream = self.stream_screenshot(img_format).await?;
//         let max_duration = std::time::Duration::from_secs(duration_secs);
//         let start = std::time::Instant::now();
//         while start.elapsed() < max_duration {
//             match video_stream.message().await {
//                 Ok(Some(frame)) => {
//                     let dt = DateTime::from_timestamp_micros(frame.timestamp_us as i64).unwrap();
//                     println!(
//                         "Received frame with timestamp: {} ,len: {}",
//                         dt,
//                         frame.image.len()
//                     );
//                 }
//                 Ok(None) => break, // stream ended
//                 Err(e) => {
//                     eprintln!("error reading video stream: {}", e);
//                     break;
//                 }
//             }
//             // Process the image (e.g., write to file or buffer)
//         }

//         Ok(())
//     }
// }

struct Recoarder {
    video_stream: Option<Streaming<Image>>,
    audio_stream: Option<Streaming<AudioPacket>>,
    output_file: PathBuf,
    is_running: Arc<AtomicBool>,
    start_time: Arc<Mutex<Option<Instant>>>,
}

impl Recoarder {
    pub fn new(
        video_stream: Option<Streaming<Image>>,
        audio_stream: Option<Streaming<AudioPacket>>,
        output_file: PathBuf,
    ) -> Self {
        Self {
            video_stream,
            audio_stream,
            output_file,
            is_running: Arc::new(AtomicBool::new(false)),
            start_time: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&mut self) {
        if !self.is_running.load(Ordering::SeqCst) {
            *self.start_time.lock().unwrap() = Some(Instant::now());
            self.is_running.store(true, Ordering::SeqCst);
            println!(
                "\x1bStarting recording to {}\x1b[0m",
                self.output_file.display()
            );
            while self.is_running.load(Ordering::SeqCst) {}
        }
    }
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        println!("\x1b[1m--------------------\nStopping recording...\x1b[0m");
    }
}
