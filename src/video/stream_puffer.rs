use crate::proto::{AudioPacket, Image};
use ffmpeg_next as ffmpeg;
use std::collections::VecDeque;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct VideoFrame {
    timestamp_ms: u32,
    data: Vec<u8>,
}

#[derive(Debug, Clone)]
struct AudioChunk {
    timestamp_ms: u32,
    data: Vec<u8>,
}

#[derive(Clone)]
pub struct StreamPuffer {
    inner: Arc<StreamPufferInner>,
}

struct StreamPufferInner {
    // ring buffers protected by RwLock for better read performance
    video_buf: RwLock<VecDeque<VideoFrame>>,
    audio_buf: RwLock<VecDeque<AudioChunk>>,
    // configuration
    max_frames: usize,
    max_audio_chunks: usize,
    // target fps and audio params (used when saving)
    target_fps: u32,
    audio_sample_rate: u32,
    audio_channels: u32,
    width: u32,
    height: u32,
}

impl StreamPuffer {
    /// Create a new puffer that retains up to `max_frames` video frames and `max_audio_chunks` audio packets.
    pub fn new(
        max_frames: usize,
        max_audio_chunks: usize,
        target_fps: u32,
        audio_sample_rate: u32,
        audio_channels: u32,
        width: u32,
        height: u32,
    ) -> Self {
        let inner = StreamPufferInner {
            video_buf: RwLock::new(VecDeque::with_capacity(max_frames)),
            audio_buf: RwLock::new(VecDeque::with_capacity(max_audio_chunks)),
            max_frames,
            max_audio_chunks,
            target_fps,
            audio_sample_rate,
            audio_channels,
            width,
            height,
        };
        Self {
            inner: Arc::new(inner),
        }
    }

    /// Push a `Image` received from the emulator into the video buffer.
    /// The `Image` is expected to be raw RGB888 bytes (as requested via ImageFormat::Rgb888).
    /// High-performance: minimizes lock time and uses pre-allocated capacity.
    pub async fn push_video(&self, img: Image) {
        let frame = VideoFrame {
            timestamp_ms: (img.timestamp_us / 1000) as u32,
            data: img.image,
        };

        let mut buf = self.inner.video_buf.write().await;
        if buf.len() >= self.inner.max_frames {
            buf.pop_front();
        }
        buf.push_back(frame);
    }

    /// Push an audio packet into the audio buffer.
    /// The `AudioPacket` is expected to contain raw PCM s16le samples (as used elsewhere in crate).
    /// High-performance: minimizes lock time and uses pre-allocated capacity.
    pub async fn push_audio(&self, pkt: AudioPacket) {
        let chunk = AudioChunk {
            timestamp_ms: (pkt.timestamp / 1000) as u32,
            data: pkt.audio,
        };

        let mut buf = self.inner.audio_buf.write().await;
        if buf.len() >= self.inner.max_audio_chunks {
            buf.pop_front();
        }
        buf.push_back(chunk);
    }

    /// Save the buffered video/audio into an MP4 file at `out_path`.
    /// Uses ffmpeg-next library for direct encoding without external processes.
    /// Performance optimized: no temp files, direct frame encoding, proper timestamp handling.
    pub async fn save_last_to_mp4(&self, out_path: impl AsRef<Path>) -> Result<(), String> {
        // Clone buffers to avoid holding locks during encoding
        let video_frames = {
            let guard = self.inner.video_buf.read().await;
            guard.iter().cloned().collect::<Vec<_>>()
        };

        let audio_chunks = {
            let guard = self.inner.audio_buf.read().await;
            guard.iter().cloned().collect::<Vec<_>>()
        };

        if video_frames.is_empty() {
            return Err("no video frames available to save".to_string());
        }

        // Calculate overlap range
        let video_start = video_frames.first().unwrap().timestamp_ms;
        let video_end = video_frames.last().unwrap().timestamp_ms;

        let (have_audio, filtered_video, filtered_audio) = if audio_chunks.is_empty() {
            // No audio: use all video frames
            (false, video_frames, Vec::new())
        } else {
            let audio_start = audio_chunks.first().unwrap().timestamp_ms;
            let audio_end = audio_chunks.last().unwrap().timestamp_ms;

            println!(
                "Video range: {} - {} ms (duration: {} ms, {} frames)",
                video_start,
                video_end,
                video_end - video_start,
                video_frames.len()
            );
            println!(
                "Audio range: {} - {} ms (duration: {} ms, {} chunks)",
                audio_start,
                audio_end,
                audio_end - audio_start,
                audio_chunks.len()
            );

            let overlap_start = video_start.max(audio_start);
            let overlap_end = video_end.min(audio_end);

            println!(
                "Overlap range: {} - {} ms (duration: {} ms)",
                overlap_start,
                overlap_end,
                overlap_end.saturating_sub(overlap_start)
            );

            // If no overlap, save video-only
            if overlap_end <= overlap_start {
                println!("No timestamp overlap found, saving video-only");
                (false, video_frames, Vec::new())
            } else {
                // Filter to overlapping frames
                let fv: Vec<_> = video_frames
                    .into_iter()
                    .filter(|f| f.timestamp_ms >= overlap_start && f.timestamp_ms <= overlap_end)
                    .collect();

                let fa: Vec<_> = audio_chunks
                    .into_iter()
                    .filter(|c| c.timestamp_ms >= overlap_start && c.timestamp_ms <= overlap_end)
                    .collect();

                println!(
                    "Filtered to {} video frames and {} audio chunks",
                    fv.len(),
                    fa.len()
                );

                (true, fv, fa)
            }
        };

        if filtered_video.is_empty() {
            return Err("no video frames available after filtering".to_string());
        }

        // Run encoding in blocking task to avoid blocking async runtime
        let out_path = out_path.as_ref().to_path_buf();
        let width = self.inner.width;
        let height = self.inner.height;
        let fps = self.inner.target_fps;
        let sample_rate = self.inner.audio_sample_rate;
        let channels = self.inner.audio_channels;

        tokio::task::spawn_blocking(move || {
            Self::encode_to_mp4(
                &out_path,
                filtered_video,
                filtered_audio,
                width,
                height,
                fps,
                sample_rate,
                channels,
                have_audio,
            )
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))??;

        Ok(())
    }

    /// Internal method to encode video/audio to MP4 using ffmpeg-next.
    /// Must be called from a blocking context (not async).
    fn encode_to_mp4(
        out_path: &Path,
        video_frames: Vec<VideoFrame>,
        audio_chunks: Vec<AudioChunk>,
        width: u32,
        height: u32,
        fps: u32,
        sample_rate: u32,
        channels: u32,
        have_audio: bool,
    ) -> Result<(), String> {
        use ffmpeg::codec;
        use ffmpeg::format;
        use ffmpeg::software::scaling;
        use ffmpeg::{frame, Rational};

        // Initialize ffmpeg once
        ffmpeg::init().map_err(|e| format!("FFmpeg init error: {}", e))?;

        // Create output context
        let path_str = out_path.to_str().ok_or("Invalid output path")?;
        let mut octx =
            format::output(&path_str).map_err(|e| format!("Cannot create output: {}", e))?;

        // --- Video Stream Setup ---
        let global_header = octx.format().flags().contains(format::Flags::GLOBAL_HEADER);

        // Use MPEG4 codec (simpler than H264, no preset requirements)
        let codec = codec::encoder::find(codec::Id::MPEG4).ok_or("MPEG4 encoder not found")?;

        let mut ost = octx
            .add_stream(codec)
            .map_err(|e| format!("Cannot add video stream: {}", e))?;
        let video_stream_index = ost.index();

        // Get encoder from codec context
        let mut video_encoder = codec::Context::new()
            .encoder()
            .video()
            .map_err(|e| format!("Cannot create video encoder: {}", e))?;

        video_encoder.set_width(width);
        video_encoder.set_height(height);
        video_encoder.set_format(ffmpeg::format::Pixel::YUV420P);
        video_encoder.set_time_base(Rational::new(1, fps as i32));
        video_encoder.set_frame_rate(Some(Rational::new(fps as i32, 1)));

        if global_header {
            video_encoder.set_flags(ffmpeg::codec::Flags::GLOBAL_HEADER);
        }

        // Open MPEG4 encoder (no preset issues)
        let mut video_encoder = video_encoder
            .open_as(codec)
            .map_err(|e| format!("Cannot open video encoder: {}", e))?;
        ost.set_parameters(&video_encoder);

        // --- Audio Stream Setup (if needed) ---
        let mut audio_encoder_opt = None;
        let mut audio_stream_idx = 0;

        if have_audio && !audio_chunks.is_empty() {
            let audio_codec =
                codec::encoder::find(codec::Id::AAC).ok_or("AAC encoder not found")?;

            let mut ast = octx
                .add_stream(audio_codec)
                .map_err(|e| format!("Cannot add audio stream: {}", e))?;
            audio_stream_idx = ast.index();

            let mut audio_enc = codec::Context::new()
                .encoder()
                .audio()
                .map_err(|e| format!("Cannot create audio encoder: {}", e))?;

            audio_enc.set_rate(sample_rate as i32);
            audio_enc.set_channel_layout(ffmpeg::ChannelLayout::default(channels as i32));
            // AAC encoder requires float planar format
            audio_enc.set_format(ffmpeg::format::Sample::F32(
                ffmpeg::format::sample::Type::Planar,
            ));
            // Use millisecond time_base for audio to match video
            audio_enc.set_time_base(Rational::new(1, 1_000));

            if global_header {
                audio_enc.set_flags(ffmpeg::codec::Flags::GLOBAL_HEADER);
            }

            let audio_enc = audio_enc
                .open_as(audio_codec)
                .map_err(|e| format!("Cannot open audio encoder: {}", e))?;
            ast.set_parameters(&audio_enc);
            audio_encoder_opt = Some(audio_enc);
        }

        // Write header
        octx.write_header()
            .map_err(|e| format!("Cannot write header: {}", e))?;

        // --- Create RGB to YUV scaler ---
        let mut scaler = scaling::Context::get(
            ffmpeg::format::Pixel::RGB24,
            width,
            height,
            ffmpeg::format::Pixel::YUV420P,
            width,
            height,
            scaling::Flags::BILINEAR,
        )
        .map_err(|e| format!("Cannot create scaler: {}", e))?;

        let first_timestamp = video_frames.first().unwrap().timestamp_ms;
        // --- Encode Video Frames ---
        for (idx, vframe) in video_frames.iter().enumerate() {
            // Create RGB frame
            let mut rgb_frame = frame::Video::new(ffmpeg::format::Pixel::RGB24, width, height);

            // Copy RGB data (assuming RGB888 format: width * height * 3 bytes)
            let expected_size = (width * height * 3) as usize;
            if vframe.data.len() != expected_size {
                eprintln!(
                    "Warning: frame {} has size {} bytes, expected {}",
                    idx,
                    vframe.data.len(),
                    expected_size
                );
                continue;
            }

            // Copy RGB data line by line respecting stride
            let stride = rgb_frame.stride(0);
            let data = rgb_frame.data_mut(0);
            for y in 0..height as usize {
                let src_offset = y * width as usize * 3;
                let dst_offset = y * stride;
                data[dst_offset..dst_offset + width as usize * 3]
                    .copy_from_slice(&vframe.data[src_offset..src_offset + width as usize * 3]);
            }

            // Convert RGB to YUV420P
            let mut yuv_frame = frame::Video::new(ffmpeg::format::Pixel::YUV420P, width, height);
            scaler
                .run(&rgb_frame, &mut yuv_frame)
                .map_err(|e| format!("Scaling error: {}", e))?;

            // Set PTS from actual frame timestamp (already in milliseconds)
            // Matches our time_base of 1/1000
            yuv_frame.set_pts(Some((vframe.timestamp_ms - first_timestamp) as i64));

            // Encode
            video_encoder
                .send_frame(&yuv_frame)
                .map_err(|e| format!("Send frame error: {}", e))?;

            // Receive packets
            let mut encoded = ffmpeg::Packet::empty();
            while video_encoder.receive_packet(&mut encoded).is_ok() {
                encoded.set_stream(video_stream_index);
                encoded.rescale_ts(
                    Rational::new(1, 1_000), // From encoder time_base (milliseconds)
                    octx.stream(video_stream_index).unwrap().time_base(),
                );
                encoded
                    .write_interleaved(&mut octx)
                    .map_err(|e| format!("Write packet error: {}", e))?;
            }
        }

        // Flush video encoder
        video_encoder
            .send_eof()
            .map_err(|e| format!("Send EOF error: {}", e))?;
        let mut encoded = ffmpeg::Packet::empty();
        while video_encoder.receive_packet(&mut encoded).is_ok() {
            encoded.set_stream(video_stream_index);
            encoded.rescale_ts(
                Rational::new(1, 1_000), // From encoder time_base (milliseconds)
                octx.stream(video_stream_index).unwrap().time_base(),
            );
            encoded
                .write_interleaved(&mut octx)
                .map_err(|e| format!("Write packet error: {}", e))?;
        }

        // --- Encode Audio (if available) ---
        if let Some(mut audio_encoder) = audio_encoder_opt {
            // AAC requires float planar (fltp) format with exactly 1024 samples per frame
            let frame_size = audio_encoder.frame_size() as usize;

            // Buffer to accumulate samples for AAC frames (interleaved f32)
            let mut sample_buffer: Vec<f32> = Vec::new();
            let mut total_samples_processed = 0usize;

            let total_audio_bytes: usize = audio_chunks.iter().map(|c| c.data.len()).sum();
            let total_audio_samples = total_audio_bytes / 2; // i16 is 2 bytes
            println!(
                "Processing {} audio chunks ({} bytes, {} samples) for AAC encoding",
                audio_chunks.len(),
                total_audio_bytes,
                total_audio_samples
            );

            for (idx, achunk) in audio_chunks.iter().enumerate() {
                // Convert s16le bytes to i16 samples, then normalize to f32 [-1.0, 1.0]
                let samples_i16: Vec<i16> = achunk
                    .data
                    .chunks_exact(2)
                    .map(|b| i16::from_le_bytes([b[0], b[1]]))
                    .collect();

                if samples_i16.is_empty() {
                    continue;
                }

                if idx < 3 {
                    println!(
                        "  Chunk {}: {} bytes -> {} samples",
                        idx,
                        achunk.data.len(),
                        samples_i16.len()
                    );
                }

                // Convert i16 to f32 and add to buffer (interleaved)
                // i16 range is -32768 to 32767, normalize to -1.0 to 1.0
                for sample in samples_i16 {
                    sample_buffer.push(sample as f32 / 32768.0);
                }

                // Process complete AAC frames
                while sample_buffer.len() >= frame_size * channels as usize {
                    let mut audio_frame = frame::Audio::new(
                        ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Planar),
                        frame_size,
                        ffmpeg::ChannelLayout::STEREO,
                    );

                    // Split interleaved buffer into planar channels
                    // Process left channel
                    {
                        let left_out = audio_frame.plane_mut::<f32>(0);
                        for i in 0..frame_size {
                            left_out[i] = sample_buffer[i * 2];
                        }
                    }
                    // Process right channel
                    {
                        let right_out = audio_frame.plane_mut::<f32>(1);
                        for i in 0..frame_size {
                            right_out[i] = sample_buffer[i * 2 + 1];
                        }
                    }

                    // Remove processed samples
                    sample_buffer.drain(0..frame_size * channels as usize);

                    // Calculate PTS based on sample position in stream
                    // Each sample represents 1/sample_rate seconds
                    let pts_ms = (total_samples_processed as i64 * 1000) / sample_rate as i64;
                    audio_frame.set_pts(Some(pts_ms));
                    total_samples_processed += frame_size;

                    // Encode audio frame
                    audio_encoder
                        .send_frame(&audio_frame)
                        .map_err(|e| format!("Send audio frame error: {}", e))?;

                    // Receive packets
                    let mut encoded = ffmpeg::Packet::empty();
                    while audio_encoder.receive_packet(&mut encoded).is_ok() {
                        encoded.set_stream(audio_stream_idx);
                        encoded.rescale_ts(
                            Rational::new(1, 1_000),
                            octx.stream(audio_stream_idx).unwrap().time_base(),
                        );
                        encoded
                            .write_interleaved(&mut octx)
                            .map_err(|e| format!("Write audio packet error: {}", e))?;
                    }
                }
            }

            // Flush audio encoder
            audio_encoder
                .send_eof()
                .map_err(|e| format!("Send audio EOF error: {}", e))?;
            let mut encoded = ffmpeg::Packet::empty();
            while audio_encoder.receive_packet(&mut encoded).is_ok() {
                encoded.set_stream(audio_stream_idx);
                encoded.rescale_ts(
                    Rational::new(1, 1_000),
                    octx.stream(audio_stream_idx).unwrap().time_base(),
                );
                encoded
                    .write_interleaved(&mut octx)
                    .map_err(|e| format!("Write audio packet error: {}", e))?;
            }
        }

        // Write trailer
        octx.write_trailer()
            .map_err(|e| format!("Cannot write trailer: {}", e))?;

        Ok(())
    }
}
