// Library wrapper around the Android EmulatorController proto

pub mod proto {
    // Generated code will be included here by tonic
    tonic::include_proto!("android.emulation.control");
}
// Re-export video submodule so external crates (tests/bins) can access it
pub mod video;
// File system operations via ADB
pub mod fs;
use tonic::transport::Channel;
use tonic::Status;

/// Configuration for screen recording
//#[derive(Debug, Clone)]
// Use the generated types through our proto module
use proto::emulator_controller_client::EmulatorControllerClient;
use proto::{
    AudioFormat, AudioPacket, BatteryState, BrightnessValue, ClipData, DisplayConfigurations,
    GpsState, Image, ImageFormat, LogMessage, PhysicalModelValue, SensorValue, Touch, TouchEvent,
    VmRunState,
};

/// Async wrapper client for the emulator controller gRPC service.
pub struct DeviceGrpcClient {
    inner: EmulatorControllerClient<Channel>,
}

impl DeviceGrpcClient {
    /// Connect to the gRPC endpoint (e.g., "127.0.0.1:8701").
    pub async fn connect(endpoint: impl Into<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let ep = endpoint.into();
        let channel = Channel::from_shared(ep)?.connect().await?;
        let inner = EmulatorControllerClient::new(channel);
        Ok(Self { inner })
    }

    /// Get clipboard text from the emulator.
    pub async fn get_clipboard(&mut self) -> Result<String, Status> {
        let req = tonic::Request::new(());
        let resp = self.inner.get_clipboard(req).await?;
        Ok(resp.into_inner().text)
    }

    /// Set clipboard text on the emulator.
    pub async fn set_clipboard(&mut self, text: impl Into<String>) -> Result<(), Status> {
        let data = ClipData { text: text.into() };
        let req = tonic::Request::new(data);
        self.inner
            .set_clipboard(req)
            .await
            .map(|_| ())
            .map_err(|e| e)
    }

    /// Send a single touch event (best-effort). This constructs a TouchEvent with a single touch.
    /// Many emulator input APIs expect sequences; this helper sends one event which often suffices for simple taps.
    pub async fn send_touch(&mut self, x: i32, y: i32) -> Result<(), Status> {
        let touch = Touch {
            x,
            y,
            identifier: 0,
            pressure: 1,
            touch_major: 0,
            touch_minor: 0,
            // expiration and orientation are enums/ints; leave as defaults (0)
            expiration: 0,
            orientation: 0,
        };
        let event = TouchEvent {
            touches: vec![touch],
            display: 0,
        };
        let req = tonic::Request::new(event);
        self.inner.send_touch(req).await.map(|_| ()).map_err(|e| e)
    }

    /// Convenience: perform a simple tap (alias to `send_touch`).
    pub async fn tap(&mut self, x: i32, y: i32) -> Result<(), Status> {
        self.send_touch(x, y).await
    }

    /// Request a continuous screenshot stream. Returns the tonic streaming of `Image`.
    pub async fn stream_screenshot(
        &mut self,
        fmt: ImageFormat,
    ) -> Result<tonic::Streaming<Image>, Status> {
        let req = tonic::Request::new(fmt);
        let resp = self.inner.stream_screenshot(req).await?;
        Ok(resp.into_inner())
    }

    /// Get a single screenshot from the emulator.
    pub async fn get_screenshot(&mut self) -> Result<Image, Status> {
        let fmt = ImageFormat {
            format: proto::image_format::ImgFormat::Png.into(),
            rotation: None,
            width: 0,
            height: 0,
            display: 0,
            transport: None,
            folded_display: None,
            display_mode: 0,
        };
        let req = tonic::Request::new(fmt);
        let resp = self.inner.get_screenshot(req).await?;
        Ok(resp.into_inner())
    }

    /// Save a screenshot as PNG file
    pub async fn save_screenshot(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let image = self.get_screenshot().await?;
        std::fs::write(path, image.image)?;
        Ok(())
    }

    /// Get the battery state from the emulator
    pub async fn get_battery(&mut self) -> Result<BatteryState, Status> {
        let req = tonic::Request::new(());
        let resp = self.inner.get_battery(req).await?;
        Ok(resp.into_inner())
    }

    /// Set the battery state on the emulator
    pub async fn set_battery(&mut self, state: BatteryState) -> Result<(), Status> {
        let req = tonic::Request::new(state);
        self.inner.set_battery(req).await.map(|_| ())
    }

    /// Get the GPS state from the emulator
    pub async fn get_gps(&mut self) -> Result<GpsState, Status> {
        let req = tonic::Request::new(());
        let resp = self.inner.get_gps(req).await?;
        Ok(resp.into_inner())
    }

    /// Set the GPS state on the emulator
    pub async fn set_gps(&mut self, state: GpsState) -> Result<(), Status> {
        let req = tonic::Request::new(state);
        self.inner.set_gps(req).await.map(|_| ())
    }

    /// Get the VM state from the emulator
    pub async fn get_vm_state(&mut self) -> Result<VmRunState, Status> {
        let req = tonic::Request::new(());
        let resp = self.inner.get_vm_state(req).await?;
        Ok(resp.into_inner())
    }

    /// Set the VM state on the emulator
    pub async fn set_vm_state(&mut self, state: VmRunState) -> Result<(), Status> {
        let req = tonic::Request::new(state);
        self.inner.set_vm_state(req).await.map(|_| ())
    }

    /// Get the display configurations from the emulator
    pub async fn get_display_configurations(&mut self) -> Result<DisplayConfigurations, Status> {
        let req = tonic::Request::new(());
        let resp = self.inner.get_display_configurations(req).await?;
        Ok(resp.into_inner())
    }

    /// Set the display configurations on the emulator
    pub async fn set_display_configurations(
        &mut self,
        configs: DisplayConfigurations,
    ) -> Result<DisplayConfigurations, Status> {
        let req = tonic::Request::new(configs);
        let resp = self.inner.set_display_configurations(req).await?;
        Ok(resp.into_inner())
    }

    /// Get the brightness value from the emulator
    pub async fn get_brightness(
        &mut self,
        value: BrightnessValue,
    ) -> Result<BrightnessValue, Status> {
        let req = tonic::Request::new(value);
        let resp = self.inner.get_brightness(req).await?;
        Ok(resp.into_inner())
    }

    /// Set the brightness value on the emulator
    pub async fn set_brightness(&mut self, value: BrightnessValue) -> Result<(), Status> {
        let req = tonic::Request::new(value);
        self.inner.set_brightness(req).await.map(|_| ())
    }

    /// Get a sensor value from the emulator
    pub async fn get_sensor(&mut self, value: SensorValue) -> Result<SensorValue, Status> {
        let req = tonic::Request::new(value);
        let resp = self.inner.get_sensor(req).await?;
        Ok(resp.into_inner())
    }

    /// Set a sensor value on the emulator
    pub async fn set_sensor(&mut self, value: SensorValue) -> Result<(), Status> {
        let req = tonic::Request::new(value);
        self.inner.set_sensor(req).await.map(|_| ())
    }

    /// Stream sensor values from the emulator
    pub async fn stream_sensor(
        &mut self,
        value: SensorValue,
    ) -> Result<tonic::Streaming<SensorValue>, Status> {
        let req = tonic::Request::new(value);
        let resp = self.inner.stream_sensor(req).await?;
        Ok(resp.into_inner())
    }

    /// Get the physical model state
    pub async fn get_physical_model(
        &mut self,
        value: PhysicalModelValue,
    ) -> Result<PhysicalModelValue, Status> {
        let req = tonic::Request::new(value);
        let resp = self.inner.get_physical_model(req).await?;
        Ok(resp.into_inner())
    }

    /// Set the physical model state
    pub async fn set_physical_model(&mut self, value: PhysicalModelValue) -> Result<(), Status> {
        let req = tonic::Request::new(value);
        self.inner.set_physical_model(req).await.map(|_| ())
    }

    /// Stream physical model values
    pub async fn stream_physical_model(
        &mut self,
        value: PhysicalModelValue,
    ) -> Result<tonic::Streaming<PhysicalModelValue>, Status> {
        let req = tonic::Request::new(value);
        let resp = self.inner.stream_physical_model(req).await?;
        Ok(resp.into_inner())
    }

    /// Stream audio from the emulator
    pub async fn stream_audio(
        &mut self,
        format: AudioFormat,
    ) -> Result<tonic::Streaming<AudioPacket>, Status> {
        let req = tonic::Request::new(format);
        let resp = self.inner.stream_audio(req).await?;
        Ok(resp.into_inner())
    }

    /// Stream logcat output
    pub async fn stream_logcat(
        &mut self,
        msg: LogMessage,
    ) -> Result<tonic::Streaming<LogMessage>, Status> {
        let req = tonic::Request::new(msg);
        let resp = self.inner.stream_logcat(req).await?;
        Ok(resp.into_inner())
    }

    /// Record audio from the emulator and save it as an MP3 file
    pub async fn record_audio(
        &mut self,
        audio_path: impl AsRef<std::path::Path>,
        duration_secs: u64,
        sample_rate: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use std::io::Write;
        use std::process::{Command, Stdio};

        // Set up audio format
        let audio_format = AudioFormat {
            sampling_rate: sample_rate as u64,
            channels: proto::audio_format::Channels::Stereo as i32,
            format: proto::audio_format::SampleFormat::AudFmtS16 as i32,
            mode: proto::audio_format::DeliveryMode::ModeUnspecified as i32,
        };

        // Start audio stream
        let mut audio_stream = self.stream_audio(audio_format).await?;

        // Bind sample_rate.to_string() to a variable to extend its lifetime
        let sample_rate_str = sample_rate.to_string();

        // Build ffmpeg args for audio
        let ffmpeg_args = vec![
            "-f",
            "s16le",
            "-ar",
            &sample_rate_str,
            "-ac",
            "2",
            "-i",
            "-", // read raw audio from stdin
            "-c:a",
            "libmp3lame",
            "-q:a",
            "2", // high-quality MP3
            audio_path.as_ref().to_str().ok_or("Invalid path")?,
        ];

        // Spawn ffmpeg process
        let mut ffmpeg = Command::new("ffmpeg")
            .args(&ffmpeg_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("failed to start ffmpeg");

        let mut ffmpeg_stdin = ffmpeg.stdin.take().expect("ffmpeg stdin");

        // Stream audio packets for the requested duration
        let start_time = std::time::Instant::now();
        while start_time.elapsed() < std::time::Duration::from_secs(duration_secs) {
            match audio_stream.message().await {
                Ok(Some(audio_packet)) => {
                    ffmpeg_stdin.write_all(&audio_packet.audio)?;
                }
                Ok(None) => break, // stream ended
                Err(e) => {
                    eprintln!("error reading audio stream: {}", e);
                    break;
                }
            }
        }

        // Close stdin to signal EOF to ffmpeg
        drop(ffmpeg_stdin);
        let status = ffmpeg.wait()?;
        println!("ffmpeg exited with: {:?}", status);

        Ok(())
    }

    /// Record screen and audio (if configured) to file
    // pub async fn record_screen(
    //     &mut self,
    //     video_path: impl AsRef<std::path::Path>,
    //     duration_secs: u64,
    //     config: Option<RecordingConfig>,
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     use std::process::Command;
    //     use tokio::io::AsyncWriteExt;
    //     use tokio::sync::mpsc;

    //     // Determine native display size from emulator
    //     let config = config.unwrap_or_default();
    //     let display_config = self.get_display_configurations().await?;
    //     let disp = display_config.displays.first().ok_or("No display found")?;
    //     let width = if config.width > 0 {
    //         config.width
    //     } else {
    //         disp.width
    //     };
    //     let height = if config.height > 0 {
    //         config.height
    //     } else {
    //         disp.height
    //     };
    //     let fps = config.fps;

    //     // Request RGB888 screenshots at the native resolution
    //     let img_format = ImageFormat {
    //         format: proto::image_format::ImgFormat::Rgb888 as i32,
    //         rotation: None,
    //         width: width,
    //         height: height,
    //         display: 0,
    //         transport: None,
    //         folded_display: None,
    //         display_mode: 0,
    //     };
    //     let video_stream = self.stream_screenshot(img_format).await?;

    //     // Set up audio stream if required
    //     let include_audio = config.include_audio;
    //     let audio_sample_rate = config.audio_sample_rate;
    //     let audio_stream_opt = if include_audio {
    //         let audio_format = AudioFormat {
    //             sampling_rate: audio_sample_rate,
    //             channels: proto::audio_format::Channels::Stereo as i32,
    //             format: proto::audio_format::SampleFormat::AudFmtS16 as i32,
    //             mode: proto::audio_format::DeliveryMode::ModeUnspecified as i32,
    //         };
    //         Some(self.stream_audio(audio_format).await?)
    //     } else {
    //         None
    //     };

    //     // Create unique FIFO paths in temp dir
    //     let tmp = std::env::temp_dir();
    //     let nanos = std::time::SystemTime::now()
    //         .duration_since(std::time::UNIX_EPOCH)
    //         .map(|d| d.as_nanos())
    //         .unwrap_or(0);
    //     let uniq = format!("ro_grpc_{}_{}", std::process::id(), nanos);
    //     let video_fifo = tmp.join(format!("video_{}.fifo", uniq));
    //     let audio_fifo = tmp.join(format!("audio_{}.fifo", uniq));

    //     // Create FIFOs (named pipes)
    //     {
    //         // use nix to create fifo with 0o600
    //         use nix::sys::stat::Mode;
    //         use nix::unistd::mkfifo;
    //         let _ = mkfifo(&video_fifo, Mode::S_IRWXU);
    //         if include_audio {
    //             let _ = mkfifo(&audio_fifo, Mode::S_IRWXU);
    //         }
    //     }

    //     // Build ffmpeg args reading from the FIFOs
    //     let mut ffmpeg_args: Vec<String> = vec![
    //         "-y".into(),
    //         "-f".into(),
    //         "rawvideo".into(),
    //         "-pix_fmt".into(),
    //         "rgb24".into(),
    //         "-s".into(),
    //         format!("{}x{}", width, height),
    //         "-r".into(),
    //         fps.to_string(),
    //         "-i".into(),
    //         video_fifo.to_str().unwrap().to_string(),
    //     ];

    //     if include_audio {
    //         ffmpeg_args.extend(vec![
    //             "-f".into(),
    //             "s16le".into(),
    //             "-ar".into(),
    //             audio_sample_rate.to_string(),
    //             "-ac".into(),
    //             "2".into(),
    //             "-i".into(),
    //             audio_fifo.to_str().unwrap().to_string(),
    //         ]);
    //     }

    //     ffmpeg_args.extend(vec![
    //         "-c:v".into(),
    //         "libx264".into(),
    //         "-pix_fmt".into(),
    //         "yuv420p".into(),
    //     ]);
    //     if include_audio {
    //         ffmpeg_args.extend(vec!["-c:a".into(), "aac".into(), "-shortest".into()]);
    //     }
    //     ffmpeg_args.push(
    //         video_path
    //             .as_ref()
    //             .to_str()
    //             .ok_or("Invalid path")?
    //             .to_string(),
    //     );

    //     // Spawn ffmpeg which reads from FIFOs
    //     let mut ffmpeg = Command::new("ffmpeg")
    //         .args(&ffmpeg_args)
    //         .stdout(std::process::Stdio::null())
    //         .stderr(std::process::Stdio::inherit())
    //         .spawn()
    //         .expect("failed to start ffmpeg");

    //     // Producer-consumer buffer for frames
    //     #[derive(Debug)]
    //     struct Frame {
    //         data: Vec<u8>,
    //         timestamp_us: u64,
    //     }

    //     let (tx, mut rx) = mpsc::channel::<Frame>(256);

    //     // Video producer: read frames from gRPC stream and push to channel
    //     let mut video_stream_producer = video_stream;
    //     let producer_handle = tokio::spawn(async move {
    //         while let Ok(Some(image)) = video_stream_producer.message().await {
    //             let f = Frame {
    //                 data: image.image,
    //                 timestamp_us: image.timestamp_us as u64,
    //             };
    //             if tx.send(f).await.is_err() {
    //                 // consumer dropped
    //                 break;
    //             }
    //         }
    //     });

    //     // Video consumer: read from channel and write to video fifo honoring timestamps
    //     let video_fifo_clone = video_fifo.clone();
    //     let video_consumer = tokio::spawn(async move {
    //         // open fifo for writing
    //         let mut file = match tokio::fs::OpenOptions::new()
    //             .write(true)
    //             .open(&video_fifo_clone)
    //             .await
    //         {
    //             Ok(f) => f,
    //             Err(e) => return Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
    //         };

    //         let mut base_ts: Option<u64> = None;
    //         let mut base_instant = std::time::Instant::now();

    //         while let Some(frame) = rx.recv().await {
    //             if base_ts.is_none() {
    //                 base_ts = Some(frame.timestamp_us);
    //                 base_instant = std::time::Instant::now();
    //             }
    //             let bt = base_ts.unwrap();
    //             let rel_us = frame.timestamp_us.saturating_sub(bt);
    //             let target_instant = base_instant + std::time::Duration::from_micros(rel_us);

    //             // convert to TokioInstant for async sleep
    //             let now = std::time::Instant::now();
    //             if target_instant > now {
    //                 let dur = target_instant - now;
    //                 tokio::time::sleep(dur).await;
    //             }

    //             if let Err(e) = file.write_all(&frame.data).await {
    //                 eprintln!("error writing video fifo: {}", e);
    //                 break;
    //             }
    //         }

    //         // close writer
    //         drop(file);
    //         Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    //     });

    //     // Audio writer task: write audio packets into audio fifo as they arrive
    //     let mut audio_writer_handle = None;
    //     if include_audio {
    //         if let Some(mut audio_stream) = audio_stream_opt {
    //             let audio_fifo_clone = audio_fifo.clone();
    //             let h = tokio::spawn(async move {
    //                 let mut file = match tokio::fs::OpenOptions::new()
    //                     .write(true)
    //                     .open(&audio_fifo_clone)
    //                     .await
    //                 {
    //                     Ok(f) => f,
    //                     Err(e) => {
    //                         return Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    //                     }
    //                 };

    //                 while let Ok(Some(packet)) = audio_stream.message().await {
    //                     if let Err(e) = file.write_all(&packet.audio).await {
    //                         eprintln!("error writing audio fifo: {}", e);
    //                         break;
    //                     }
    //                 }

    //                 drop(file);
    //                 Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    //             });
    //             audio_writer_handle = Some(h);
    //         }
    //     }

    //     // Wait for the requested duration, then stop producing frames
    //     let max_duration = std::time::Duration::from_secs(duration_secs);
    //     tokio::time::sleep(max_duration).await;

    //     // stop producer by dropping channel sender (producer will exit when it attempts to send)
    //     // Note: producer owns tx through move; to drop it we abort the producer task
    //     producer_handle.abort();

    //     // Close the sending side by dropping tx (it was moved into producer), ensure consumer drains
    //     // Wait for consumer tasks to finish
    //     let _ = video_consumer.await;
    //     if let Some(h) = audio_writer_handle {
    //         let _ = h.await;
    //     }

    //     // Give ffmpeg a moment to finalize and then wait for it
    //     // Dropping writers already signaled EOF on FIFOs
    //     let status = ffmpeg.wait()?;
    //     println!("ffmpeg exited with: {:?}", status);

    //     // Cleanup FIFOs
    //     let _ = std::fs::remove_file(&video_fifo);
    //     if include_audio {
    //         let _ = std::fs::remove_file(&audio_fifo);
    //     }

    //     Ok(())
    // }

    /// Save logcat output to a file for a specified duration
    pub async fn save_logcat(
        &mut self,
        file_path: impl AsRef<std::path::Path>,
        duration_secs: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs::File;
        use std::io::Write;
        use std::time::Duration;
        use tokio::time::sleep;

        let msg = LogMessage {
            contents: String::new(),
            #[allow(deprecated)]
            start: 0,
            #[allow(deprecated)]
            next: 0,
            sort: proto::log_message::LogType::Parsed as i32,
            entries: Vec::new(),
        };

        let mut logcat_stream = self.stream_logcat(msg).await?;
        let mut file = File::create(file_path)?;
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < Duration::from_secs(duration_secs) {
            if let Ok(Some(log_msg)) = logcat_stream.message().await {
                writeln!(file, "{}", log_msg.contents)?;
                for entry in log_msg.entries {
                    writeln!(
                        file,
                        "[{}] {} ({}/{}): {}",
                        entry.level, entry.tag, entry.pid, entry.tid, entry.msg
                    )?;
                }
            }
            sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    pub async fn recoard_video(
        &mut self,
        duration_secs: u64,
        custom_config: Option<RecordingConfig>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use chrono::DateTime;
        // retreave display config to get native resolution
        let mut config = custom_config.unwrap_or_default();
        if config.width == 0 || config.height == 0 {
            let displays_config = self.get_display_configurations().await?;
            let main_display = displays_config.displays.first().ok_or("No display found")?;
            config.width = main_display.width;
            config.height = main_display.height;
        }

        let img_format = ImageFormat {
            format: proto::image_format::ImgFormat::Rgb888 as i32,
            rotation: None,
            width: config.width,
            height: config.height,
            display: config.display,
            transport: None,
            folded_display: None,
            display_mode: 0,
        };
        let mut video_stream = self.stream_screenshot(img_format).await?;
        let max_duration = std::time::Duration::from_secs(duration_secs);
        let start = std::time::Instant::now();
        while start.elapsed() < max_duration {
            match video_stream.message().await {
                Ok(Some(frame)) => {
                    let dt = DateTime::from_timestamp_micros(frame.timestamp_us as i64).unwrap();
                    println!(
                        "Received frame with timestamp: {} ,len: {}",
                        dt,
                        frame.image.len()
                    );
                }
                Ok(None) => break, // stream ended
                Err(e) => {
                    eprintln!("error reading video stream: {}", e);
                    break;
                }
            }
            // Process the image (e.g., write to file or buffer)
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RecordingConfig {
    /// Whether to include audio in the recording
    pub include_audio: bool,
    /// Frame rate for video capture (frames per second)
    pub fps: u32,
    /// Width of the captured video (0 for native resolution)
    pub width: u32,
    /// Height of the captured video (0 for native resolution)
    pub height: u32,
    /// Display index to record from (0 for main display)
    pub display: u32,
    /// Audio sample rate (Hz), only used if include_audio is true
    pub audio_sample_rate: u64,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            include_audio: false,
            fps: 30,
            width: 0,
            height: 0,
            display: 0,
            audio_sample_rate: 44100,
        }
    }
}
