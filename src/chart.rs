use crate::HEIGHT;
use crate::*;
use serde_derive::Deserialize;
use std::fs;

trait Within {
    fn within(&self, d: Self, i: Self) -> bool;
}

impl Within for f32 {
    fn within(&self, d: Self, i: Self) -> bool {
        (self - d..self + d).contains(&i)
    }
}

// charts
#[derive(Deserialize, Debug, Clone)]
pub struct Chart {
    pub title: String,
    pub artist: String,
    #[serde(skip_deserializing)]
    pub filename: String,
    pub bpm: u16,
    pub duration: u32,
    pub notes: Vec<Note>,
}

impl Chart {
    pub fn open(filename: String) -> Self {
        let mut chart: Self = serde_json::from_str(
            &fs::read_to_string(format!("songs/{}/{}.json", filename, filename)).unwrap(),
        )
        .unwrap();
        chart.filename = filename;
        chart
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Note {
    pub time: u32,
    pub key: usize,
    #[serde(skip_deserializing)]
    hit: bool,
}

impl Note {
    fn appear_time(&self, _bpm: u16) -> u32 {
        self.time - 1000
    }

    fn bottom_time(&self) -> u32 {
        self.time
    }

    pub fn height_at(&self, bpm: u16, time: u32) -> i64 {
        let mut time = time as i64;
        time -= self.appear_time(bpm) as i64;
        (HEIGHT as f32 - HEIGHT as f32 * time as f32 / 1000.0) as i64
    }

    pub fn update_stats(&mut self, stats: &mut Stats, time: u32, keys: &[bool; 4]) -> bool {
        if self.hit {
            return false;
        }
        if (time.max(100) - 100..time + 100).contains(&self.time) && keys[self.key] {
            stats.score += (time as i64 - self.time as i64).abs() as u32;
            stats.combo += 1;
            if stats.combo > stats.best_combo { stats.best_combo = stats.combo }
            self.hit = true;
            stats.notes += 1;
            stats.acc = stats.acc + ((1.0 - stats.acc) / stats.notes as f32);
            return true;
        } else if time > self.time && !self.hit {
            self.hit = true;
            stats.combo = 0;
            stats.notes += 1;
            stats.acc = stats.acc + (-stats.acc / stats.notes as f32);
            return false;
        } else {
            false
        }
    }

    pub fn draw(&self, buffer: &mut Vec<u32>, bpm: u16, time: u32) {
        let height = self.height_at(bpm, time);

        if height < 0 {
            return;
        }
        if height > HEIGHT as i64 {
            return;
        }

        draw_key(buffer, self.key, height as u32, 0xc5c8c6);
    }
}

pub fn charts() -> Vec<Chart> {
    let filenames = fs::read_dir("songs")
        .unwrap()
        .map(|f| f.unwrap().file_name());

    let mut v = Vec::new();
    for filename in filenames {
        v.push(Chart::open(filename.into_string().unwrap()))
    }
    v
}
