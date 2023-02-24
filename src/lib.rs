use crate::utils::*;
use std::cmp::Ordering;
use std::time::SystemTime;

mod utils;

// Requirements Specification
// 1. ID must be a 64-bit unsigned integer
// 2. ID must be unique
// 3. ID mut can be sorted by time
//
// ┌────────timestamp(42bit)──────────┬──sequence(10bit)───┬───serial(12bit)───┐
// │                                  │                    │                   │
// │                                  │                    │                   │
// └──────────────────────────────────┴ total 64 bits──────┴───────────────────┘

const MAX_IDS_PER_MILLISECOND: usize = 4096;

#[derive(Debug, Clone, Copy)]
pub struct IdGenerator {
    epoch: SystemTime,
    timestamp: i64,
    machine_id: i32,
    server_id: i32,
    index: usize,
}

impl IdGenerator {
    pub fn new(machine_id: i32, server_id: i32) -> Self {
        let epoch = get_epoch();

        Self::with_epochs(machine_id, server_id, epoch)
    }

    fn with_epochs(machine_id: i32, server_id: i32, epoch: SystemTime) -> Self {
        let timestamp = get_timestamp(epoch);

        Self {
            epoch,
            timestamp,
            machine_id,
            server_id,
            index: 0,
        }
    }

    pub fn generate_id(&mut self) -> i64 {
        self.index = self.generalize_index(self.index);

        if self.index == 0 {
            let mut now = get_timestamp(self.epoch);

            if now == self.timestamp {
                now = bind_time(self.timestamp, self.epoch);
            }

            self.timestamp = now;
        }

        self.shift_bits(
            self.timestamp, 
            self.machine_id, 
            self.server_id, 
            self.index
        )
    }

    /// generate a unique id by using real time
    pub fn generate_id_by_time(&mut self) -> i64 {
        self.index = self.generalize_index(self.index);

        let mut now = get_timestamp(self.epoch);

        match now.cmp(&self.timestamp) {
            Ordering::Equal => {
                if self.index == 0 {
                    now = bind_time(now, self.epoch);
                    self.timestamp = now;
                }
            }
            _ => {
                self.timestamp = now;
                self.index = 0;
            }
        }

        self.shift_bits(
            self.timestamp,
            self.machine_id,
            self.server_id,
            self.index,
        )
    }

    pub fn generate_id_lazy(&mut self) -> i64 {
        self.index = self.generalize_index(self.index);

        if self.index == 0 {
            self.timestamp += 1;
        }

        self.shift_bits(
            self.timestamp,
            self.machine_id,
            self.server_id,
            self.index,
        )
    }

    /// helper function to generate id
    fn shift_bits(&self, timestamp: i64, machine_id: i32, server_id: i32, index: usize) -> i64 {
        // `self.timestamp` is 64 bits, left shift 22 bits to make it 42 bits
        // `self.machine_id` left shift 17 bits to make it 12 bits
        // `self.server_id` left shift 12 bits to make it 12 bits
        // `self.index` is complementing bits.
        timestamp << 22
        | (machine_id as i64) << 17
        | (server_id as i64) << 12
        | index as i64
    }

    fn generalize_index(&mut self, index: usize) -> usize {
        // because we have 12 bits for serial number, which means we can generate 4096 ids in one millisecond
        // so need to divide the time into 4096 parts.
        (index + 1) % MAX_IDS_PER_MILLISECOND
    }
}

#[derive(Debug, Clone)]
pub struct IdGeneratorBucket {
    id_gen: IdGenerator,
    bucket: Vec<i64>,
}

impl IdGeneratorBucket {
    pub fn new(machine_id: i32, server_id: i32) -> Self {
        let epoch = get_epoch();
        Self::with_epochs(machine_id, server_id, epoch)
    }

    fn with_epochs(machine_id: i32, server_id: i32, epoch: SystemTime) -> Self {
        let id_gen = IdGenerator::with_epochs(machine_id, server_id, epoch);
        let bucket = Vec::with_capacity(MAX_IDS_PER_MILLISECOND);

        Self { id_gen, bucket }
    }

    pub fn get_id(&mut self) -> i64 {
        if self.bucket.is_empty() {
            self.generate_ids();
        }

        self.bucket.pop().unwrap()
    }

    pub fn generate_ids(&mut self) {
        for _ in 0..MAX_IDS_PER_MILLISECOND {
            self.bucket.push(self.id_gen.generate_id_lazy());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    const MAX_CAPACITY: usize = 10_000;

    #[test]
    fn test_id_generator_real_time() {
        let now = Instant::now();

        let mut id_gen = IdGenerator::new(1, 2);
        let mut ids: Vec<i64> = Vec::with_capacity(MAX_CAPACITY);

        for _ in 0..99 {
            for _ in 0..MAX_CAPACITY {
                ids.push(id_gen.generate_id_by_time());
            }

            ids.sort();
            ids.dedup();

            assert_eq!(ids.len(), MAX_CAPACITY);

            println!("{}", ids[9999]);

            ids.clear();
        }
        println!("time elapsed: {:?}\n", now.elapsed());
    }

    #[test]
    fn test_generate_id_basic() {
        let now = Instant::now();

        let mut id_gen = IdGenerator::new(1, 2);
        let mut ids: Vec<i64> = Vec::with_capacity(MAX_CAPACITY);

        for _ in 0..99 {
            for _ in 0..MAX_CAPACITY {
                ids.push(id_gen.generate_id());
            }
            ids.sort();
            ids.dedup();

            assert_eq!(ids.len(), MAX_CAPACITY);

            println!("{}", ids[9999]);

            ids.clear();
        }

        println!("time elapsed: {:?}\n", now.elapsed());
    }

    #[test]
    fn test_lazy_generate() {
        let now = Instant::now();

        let mut id_gen = IdGenerator::new(1, 2);
        let mut ids: Vec<i64> = Vec::with_capacity(MAX_CAPACITY);

        for _ in 0..99 {
            for _ in 0..MAX_CAPACITY {
                ids.push(id_gen.generate_id_lazy());
            }

            ids.sort();
            ids.dedup();

            assert_eq!(ids.len(), MAX_CAPACITY);
            println!("{}", ids[9999]);

            ids.clear();
        }

        println!("time elapsed: {:?}\n", now.elapsed());
    }
}
