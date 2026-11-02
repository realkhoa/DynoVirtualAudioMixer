//! @file ring_buffer.rs
//! @brief Lock-free Single-Producer Single-Consumer (SPSC) audio ring buffer implementations.
//! @details Provides non-blocking, wait-free sample streaming (`AudioRingBuffer`) and chunk queuing
//! (`ChunkRingBuffer`) using the `rtrb` real-time lock-free ring buffer for glitch-free audio transfer
//! between WASAPI capture threads and playback/mixing threads.

use super::types::AudioChunk;

/// @brief Producer end of a lock-free floating-point audio sample ring buffer.
pub struct AudioProducer {
    producer: rtrb::Producer<f32>,
    capacity: usize,
}

impl AudioProducer {
    /// @brief Pushes a slice of 32-bit floating-point audio samples into the ring buffer without blocking.
    /// @param samples Slice of interleaved audio samples to write.
    /// @return Number of samples successfully written (less than `samples.len()` if buffer became full).
    pub fn push_samples(&mut self, samples: &[f32]) -> usize {
        let mut count = 0;
        for &sample in samples {
            if self.producer.push(sample).is_ok() {
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    /// @brief Pushes samples from an `AudioChunk` into the ring buffer without blocking.
    /// @param chunk Reference to `AudioChunk` whose samples should be pushed.
    /// @return Number of samples successfully written.
    #[inline]
    pub fn push_chunk(&mut self, chunk: &AudioChunk) -> usize {
        self.push_samples(&chunk.samples)
    }

    /// @brief Returns the number of available empty slots for writing.
    /// @return Number of free sample slots remaining.
    #[inline]
    pub fn slots(&self) -> usize {
        self.producer.slots()
    }

    /// @brief Checks whether the ring buffer is full and cannot accept more samples.
    /// @return `true` if full, `false` otherwise.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.producer.is_full()
    }

    /// @brief Returns the total allocated capacity of the ring buffer.
    /// @return Capacity in samples.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// @brief Consumer end of a lock-free floating-point audio sample ring buffer.
pub struct AudioConsumer {
    consumer: rtrb::Consumer<f32>,
    capacity: usize,
}

impl AudioConsumer {
    /// @brief Reads audio samples from the ring buffer into the destination slice without blocking.
    /// @param dest Destination slice to fill with audio samples.
    /// @return Number of samples successfully read (less than `dest.len()` if buffer was exhausted).
    pub fn pop_samples(&mut self, dest: &mut [f32]) -> usize {
        let mut count = 0;
        for slot in dest.iter_mut() {
            if let Ok(sample) = self.consumer.pop() {
                *slot = sample;
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    /// @brief Attempts to read a full `AudioChunk` of the specified frame count.
    /// @param frame_count Number of audio frames to retrieve.
    /// @param channels Channel count (1 for Mono, 2 for Stereo).
    /// @param sample_rate Sampling rate in Hertz.
    /// @return `Some(AudioChunk)` if sufficient samples are available, or `None` if buffer has fewer samples.
    pub fn pop_chunk(
        &mut self,
        frame_count: usize,
        channels: u16,
        sample_rate: u32,
    ) -> Option<AudioChunk> {
        let needed = frame_count.saturating_mul(channels as usize);
        if self.consumer.slots() < needed {
            return None;
        }

        let mut samples = vec![0.0f32; needed];
        let read = self.pop_samples(&mut samples);
        if read == needed {
            Some(AudioChunk::new(samples, channels, sample_rate))
        } else {
            None
        }
    }

    /// @brief Returns the number of readable samples currently buffered.
    /// @return Number of available samples.
    #[inline]
    pub fn slots(&self) -> usize {
        self.consumer.slots()
    }

    /// @brief Checks whether the ring buffer contains zero samples.
    /// @return `true` if empty, `false` otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.consumer.is_empty()
    }

    /// @brief Returns the total allocated capacity of the ring buffer.
    /// @return Capacity in samples.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// @brief Real-time lock-free SPSC ring buffer for floating-point audio samples.
/// @details Wraps `rtrb` lock-free ring buffer and provides unified or split producer/consumer
/// access patterns for thread-safe audio streaming between WASAPI callbacks.
pub struct AudioRingBuffer {
    producer: AudioProducer,
    consumer: AudioConsumer,
    capacity: usize,
}

impl AudioRingBuffer {
    /// @brief Allocates a new lock-free audio ring buffer with the given sample capacity.
    /// @param capacity Maximum number of `f32` audio samples the buffer can hold.
    /// @return A new `AudioRingBuffer` instance.
    pub fn new(capacity: usize) -> Self {
        let (producer, consumer) = rtrb::RingBuffer::new(capacity);
        Self {
            producer: AudioProducer { producer, capacity },
            consumer: AudioConsumer { consumer, capacity },
            capacity,
        }
    }

    /// @brief Splits the ring buffer into independent `AudioProducer` and `AudioConsumer` endpoints.
    /// @details Allows transferring the producer to an audio capture thread and the consumer
    /// to an audio mixer or playback thread.
    /// @return A tuple of `(AudioProducer, AudioConsumer)`.
    pub fn split(self) -> (AudioProducer, AudioConsumer) {
        (self.producer, self.consumer)
    }

    /// @brief Pushes audio samples into the buffer without blocking.
    /// @param samples Slice of audio samples to append.
    /// @return Number of samples successfully written.
    #[inline]
    pub fn push_samples(&mut self, samples: &[f32]) -> usize {
        self.producer.push_samples(samples)
    }

    /// @brief Reads audio samples into the destination slice without blocking.
    /// @param dest Target buffer for read samples.
    /// @return Number of samples read.
    #[inline]
    pub fn pop_samples(&mut self, dest: &mut [f32]) -> usize {
        self.consumer.pop_samples(dest)
    }

    /// @brief Pushes all samples from an `AudioChunk` into the buffer.
    /// @param chunk Audio chunk to write.
    /// @return Number of samples successfully written.
    #[inline]
    pub fn push_chunk(&mut self, chunk: &AudioChunk) -> usize {
        self.producer.push_chunk(chunk)
    }

    /// @brief Pops a complete `AudioChunk` from the buffer if sufficient samples exist.
    /// @param frame_count Desired frame count.
    /// @param channels Desired channel count.
    /// @param sample_rate Sample rate in Hertz.
    /// @return `Some(AudioChunk)` if successful, `None` if insufficient samples.
    #[inline]
    pub fn pop_chunk(
        &mut self,
        frame_count: usize,
        channels: u16,
        sample_rate: u32,
    ) -> Option<AudioChunk> {
        self.consumer.pop_chunk(frame_count, channels, sample_rate)
    }

    /// @brief Returns the total allocated capacity of the ring buffer in samples.
    /// @return Maximum sample count.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// @brief Returns the number of empty slots available for writing.
    /// @return Free slot count.
    #[inline]
    pub fn slots(&self) -> usize {
        self.producer.slots()
    }

    /// @brief Checks if the buffer contains zero readable samples.
    /// @return `true` if empty, `false` otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.consumer.is_empty()
    }

    /// @brief Checks if the buffer is completely full.
    /// @return `true` if full, `false` otherwise.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.producer.is_full()
    }
}

/// @brief Producer end of a lock-free discrete `AudioChunk` ring buffer.
pub struct ChunkProducer {
    producer: rtrb::Producer<AudioChunk>,
    capacity: usize,
}

impl ChunkProducer {
    /// @brief Pushes an `AudioChunk` into the queue without blocking.
    /// @param chunk The audio chunk to push.
    /// @return `Ok(())` on success, or `Err(AudioChunk)` containing the unwritten chunk if the queue is full.
    pub fn push(&mut self, chunk: AudioChunk) -> Result<(), AudioChunk> {
        self.producer.push(chunk).map_err(|err| match err {
            rtrb::PushError::Full(val) => val,
        })
    }

    /// @brief Returns the number of empty chunk slots remaining.
    /// @return Available write slots.
    #[inline]
    pub fn slots(&self) -> usize {
        self.producer.slots()
    }

    /// @brief Checks whether the chunk queue is full.
    /// @return `true` if full, `false` otherwise.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.producer.is_full()
    }

    /// @brief Returns the total allocated capacity in chunk count.
    /// @return Maximum number of chunks buffer can hold.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// @brief Consumer end of a lock-free discrete `AudioChunk` ring buffer.
pub struct ChunkConsumer {
    consumer: rtrb::Consumer<AudioChunk>,
    capacity: usize,
}

impl ChunkConsumer {
    /// @brief Pops the next `AudioChunk` from the queue without blocking.
    /// @return `Some(AudioChunk)` if available, `None` if empty.
    #[inline]
    pub fn pop(&mut self) -> Option<AudioChunk> {
        self.consumer.pop().ok()
    }

    /// @brief Returns the number of chunks ready to be consumed.
    /// @return Number of buffered chunks.
    #[inline]
    pub fn slots(&self) -> usize {
        self.consumer.slots()
    }

    /// @brief Checks whether the chunk queue is empty.
    /// @return `true` if empty, `false` otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.consumer.is_empty()
    }

    /// @brief Returns the total allocated capacity in chunk count.
    /// @return Maximum number of chunks buffer can hold.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// @brief Lock-free SPSC ring buffer for discrete `AudioChunk` structures.
pub struct ChunkRingBuffer {
    producer: ChunkProducer,
    consumer: ChunkConsumer,
    capacity: usize,
}

impl ChunkRingBuffer {
    /// @brief Creates a new `ChunkRingBuffer` with capacity for `capacity` complete audio chunks.
    /// @param capacity Number of `AudioChunk` items buffer can store.
    /// @return A new `ChunkRingBuffer` instance.
    pub fn new(capacity: usize) -> Self {
        let (producer, consumer) = rtrb::RingBuffer::new(capacity);
        Self {
            producer: ChunkProducer { producer, capacity },
            consumer: ChunkConsumer { consumer, capacity },
            capacity,
        }
    }

    /// @brief Splits the chunk ring buffer into independent `ChunkProducer` and `ChunkConsumer` endpoints.
    /// @return A tuple of `(ChunkProducer, ChunkConsumer)`.
    pub fn split(self) -> (ChunkProducer, ChunkConsumer) {
        (self.producer, self.consumer)
    }

    /// @brief Pushes an `AudioChunk` into the queue without blocking.
    /// @param chunk The audio chunk to push.
    /// @return `Ok(())` on success, `Err(AudioChunk)` if queue is full.
    #[inline]
    pub fn push(&mut self, chunk: AudioChunk) -> Result<(), AudioChunk> {
        self.producer.push(chunk)
    }

    /// @brief Pops the next `AudioChunk` from the queue without blocking.
    /// @return `Some(AudioChunk)` if available, `None` if empty.
    #[inline]
    pub fn pop(&mut self) -> Option<AudioChunk> {
        self.consumer.pop()
    }

    /// @brief Returns the total chunk capacity.
    /// @return Maximum number of chunks.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// @brief Returns available write slots for new chunks.
    /// @return Number of free chunk slots.
    #[inline]
    pub fn slots(&self) -> usize {
        self.producer.slots()
    }

    /// @brief Checks if the chunk queue contains zero chunks.
    /// @return `true` if empty, `false` otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.consumer.is_empty()
    }

    /// @brief Checks if the chunk queue is full.
    /// @return `true` if full, `false` otherwise.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.producer.is_full()
    }
}
