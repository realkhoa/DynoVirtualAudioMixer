//! @file test_ring_buffer.rs
//! @brief Integration and unit tests for lock-free audio ring buffer and audio chunk primitives.
//! @details Validates capacity allocation, SPSC sample streaming, AudioChunk lifecycle,
//! peak decibel calculation (-60.0 dB floor to 0.0 dB full scale), soft clipping,
//! and lock-free thread safety under high throughput.

use dyno_virtual_audio_mixer_lib::audio::ring_buffer::{AudioRingBuffer, ChunkRingBuffer};
use dyno_virtual_audio_mixer_lib::audio::types::{
    calculate_peak_db, AudioChunk, ChannelCount, SampleRate, MIN_PEAK_DB,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

/// @brief Tests creation, capacity reporting, and empty/full states of AudioRingBuffer.
#[test]
fn test_ring_buffer_creation_and_capacity() {
    let capacity = 1024;
    let ring_buffer = AudioRingBuffer::new(capacity);

    assert_eq!(ring_buffer.capacity(), capacity);
    assert_eq!(ring_buffer.slots(), capacity);
    assert!(ring_buffer.is_empty());
    assert!(!ring_buffer.is_full());
}

/// @brief Tests pushing and popping f32 audio samples through AudioRingBuffer in SPSC fashion.
#[test]
fn test_spsc_push_and_pop_samples() {
    let mut ring_buffer = AudioRingBuffer::new(16);
    let input_samples = vec![0.1f32, 0.2, 0.3, 0.4, 0.5, 0.6];
    
    let written = ring_buffer.push_samples(&input_samples);
    assert_eq!(written, input_samples.len());
    assert_eq!(ring_buffer.slots(), 16 - written);

    let mut output_samples = vec![0.0f32; 6];
    let read = ring_buffer.pop_samples(&mut output_samples);
    assert_eq!(read, 6);
    for (i, (&in_val, &out_val)) in input_samples.iter().zip(output_samples.iter()).enumerate() {
        assert!((in_val - out_val).abs() < 1e-6, "Mismatch at sample {}", i);
    }
}

/// @brief Tests AudioChunk creation, silence generation, and sample rate / channel metadata.
#[test]
fn test_audio_chunk_metadata_and_silence() {
    let channels = ChannelCount::Stereo;
    let sample_rate = SampleRate::RATE_48000;
    let chunk = AudioChunk::silence(256, channels.channels(), sample_rate.hz());

    assert_eq!(chunk.channels, 2);
    assert_eq!(chunk.sample_rate, 48000);
    assert_eq!(chunk.frame_count(), 256);
    assert_eq!(chunk.samples.len(), 512);
    assert!(chunk.samples.iter().all(|&s| s == 0.0));
}

/// @brief Tests peak dB calculation for silence (-60 dB) and full scale (0 dB).
#[test]
fn test_peak_db_silence_and_full_scale() {
    // 1. Absolute silence should yield MIN_PEAK_DB (-60.0 dB)
    let silence = AudioChunk::silence(128, 2, 48000);
    let (left_db, right_db) = silence.calculate_peak_db();
    assert_eq!(left_db, MIN_PEAK_DB);
    assert_eq!(right_db, MIN_PEAK_DB);

    // 2. Full scale (+1.0 and -1.0) should yield exactly 0.0 dB
    let full_scale = AudioChunk::new(vec![1.0, -1.0, 0.5, -0.5], 2, 48000);
    let (l_peak, r_peak) = full_scale.calculate_peak_db();
    assert!((l_peak - 0.0).abs() < 1e-4, "Left peak should be 0.0 dB, got {}", l_peak);
    assert!((r_peak - 0.0).abs() < 1e-4, "Right peak should be 0.0 dB, got {}", r_peak);

    // 3. Standalone function calculate_peak_db
    let (standalone_l, standalone_r) = calculate_peak_db(&[0.5, 0.0], 2);
    // 20 * log10(0.5) = -6.0205999 dB
    assert!((standalone_l - (-6.0206)).abs() < 1e-3, "Expected -6.02 dB, got {}", standalone_l);
    assert_eq!(standalone_r, MIN_PEAK_DB);
}

/// @brief Tests soft clipping limiter on AudioChunk to prevent hard digital clipping.
#[test]
fn test_audio_chunk_soft_clipping() {
    let mut chunk = AudioChunk::new(vec![0.5, -0.5, 2.0, -3.0], 2, 48000);
    chunk.soft_clip();

    // Values <= 1.0 should stay close to their tanh curve and not exceed 1.0
    for &sample in &chunk.samples {
        assert!(sample.abs() < 1.0, "Sample {} exceeded 1.0 after soft clipping", sample);
    }
    // Check tanh(2.0) is approx 0.9640
    assert!((chunk.samples[2] - 2.0f32.tanh()).abs() < 1e-4);
}

/// @brief Tests ChunkRingBuffer single-producer single-consumer pushing and popping of AudioChunk.
#[test]
fn test_chunk_ring_buffer_push_pop() {
    let mut chunk_rb = ChunkRingBuffer::new(4);
    assert_eq!(chunk_rb.capacity(), 4);

    let chunk1 = AudioChunk::new(vec![0.1, 0.2], 2, 48000);
    let chunk2 = AudioChunk::new(vec![0.3, 0.4], 2, 48000);

    assert!(chunk_rb.push(chunk1.clone()).is_ok());
    assert!(chunk_rb.push(chunk2.clone()).is_ok());

    let popped1 = chunk_rb.pop().expect("Expected chunk 1");
    assert_eq!(popped1.samples, chunk1.samples);

    let popped2 = chunk_rb.pop().expect("Expected chunk 2");
    assert_eq!(popped2.samples, chunk2.samples);

    assert!(chunk_rb.pop().is_none());
}

/// @brief Tests multi-threaded lock-free streaming across producer and consumer threads without drop or deadlock.
#[test]
fn test_multithreaded_producer_consumer() {
    let total_samples = 48000;
    let ring_buffer = AudioRingBuffer::new(4096);
    let (mut producer, mut consumer) = ring_buffer.split();

    let done = Arc::new(AtomicBool::new(false));
    let done_clone = done.clone();

    // Producer thread: sends sequential sine or counter samples in chunks
    let producer_handle = thread::spawn(move || {
        let chunk_size = 256;
        let mut sent = 0;
        let mut buffer = vec![0.0f32; chunk_size];

        while sent < total_samples {
            let to_send = chunk_size.min(total_samples - sent);
            for (i, sample) in buffer.iter_mut().enumerate().take(to_send) {
                *sample = (sent + i) as f32;
            }

            let mut offset = 0;
            while offset < to_send {
                let pushed = producer.push_samples(&buffer[offset..to_send]);
                if pushed > 0 {
                    offset += pushed;
                } else {
                    thread::yield_now();
                }
            }
            sent += to_send;
        }

        done_clone.store(true, Ordering::Release);
    });

    // Consumer thread: reads samples and verifies monotonic order
    let consumer_handle = thread::spawn(move || {
        let mut received = 0;
        let mut read_buf = vec![0.0f32; 128];

        while received < total_samples {
            let read = consumer.pop_samples(&mut read_buf);
            if read > 0 {
                for (i, &actual) in read_buf.iter().enumerate().take(read) {
                    let expected = (received + i) as f32;
                    assert_eq!(actual, expected, "Mismatch at sample index {}", received + i);
                }
                received += read;
            } else if done.load(Ordering::Acquire) && consumer.is_empty() {
                break;
            } else {
                thread::yield_now();
            }
        }

        assert_eq!(received, total_samples, "Did not receive all transmitted samples");
    });

    producer_handle.join().expect("Producer thread panicked");
    consumer_handle.join().expect("Consumer thread panicked");
}
