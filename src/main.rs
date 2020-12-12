mod drawing;
mod chart;

use minifb::{Key, Window, WindowOptions, KeyRepeat};
use bdf::Font;
use drawing::*;
use chart::*;
use std::time::{Instant, Duration};
use std::fs;
use std::collections::HashMap;
use std::io::{BufReader, Cursor};
use std::sync::Arc;
use rodio::Source;
use image::io::Reader as ImageReader;
use image::ImageFormat;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

fn pick_chart(buffer: &mut Vec<u32>, window: &mut Window, charts: &Vec<Chart>, font: &Font) -> usize {
    let mut selected = 0usize;

    let titles: String = charts
                    .iter()
                    .map(|n| n.title.clone())
                    .collect::<Vec<_>>()
                    .join("\n");

    loop {
        *buffer = vec![0; WIDTH * HEIGHT];
        draw_string(buffer, font, 30, 20, 0xffaaaa, &titles);

        for key in window.get_keys_pressed(KeyRepeat::No).unwrap_or(Vec::new()) {
            match key {
                Key::Enter => return selected,
                Key::Down => if selected != charts.len() - 1 { selected += 1 },
                Key::Up => if selected != 0 { selected -= 1 },
                _ => {}
            }
        }

        let picker_string = { let mut s = "\n".repeat(selected); s.push('>'); s };
        draw_string(buffer, font, 10, 20, 0xaaffff, &picker_string);

        let _ = window.update_with_buffer(buffer, WIDTH, HEIGHT);
    }
}

// gameplay !

#[derive(Debug)]
pub struct Stats {
    acc: f32,
    score: u32,
    combo: u32,
    best_combo: u32,
    notes: u32,
}

impl ToString for Stats {
    fn to_string(&self) -> String {
        format!("acc: {:.2}\nscr: {}\ncmb: {}", self.acc * 100.0, self.score, self.combo)
    }
}

fn play(buffer: &mut Vec<u32>, window: &mut Window, chart: &mut Chart, font: &Font) -> Stats {
    let mut stats = Stats { acc: 0.0, score: 0, combo: 0, best_combo: 0, notes: 0 };

    let mut chart = chart.clone();
    let mut notes = chart.notes.clone();

    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    let song = fs::File::open(format!("songs/{0}/{0}.ogg", chart.filename)).unwrap();
    let source = rodio::Decoder::new(BufReader::new(song)).unwrap();
    stream_handle.play_raw(source.convert_samples());

    let hitsound = fs::read("res/hitsound.wav").unwrap();
    let hitsound_arc: Arc<[u8]> = hitsound.into();

    let now = Instant::now();
    let mut time = 0u32;

    while window.is_open() && !(time > chart.duration) {
        *buffer = vec![0; WIDTH * HEIGHT];
        draw_string(buffer, font, ((WIDTH / 2) - 20) as u32, (HEIGHT / 2) as u32, 0xffffff, &stats.to_string());
        draw_string(buffer, font, 0, 0, 0xaaaaaa, &format!("{}ms",  now.elapsed().as_millis() as u32 - time));

        time = now.elapsed().as_millis() as u32;

        // draws currently pressed keys
        for key in window.get_keys().unwrap_or(Vec::new()) {
            let n = match key {
                Key::Left => 0,
                Key::Down => 1,
                Key::Up => 2,
                Key::Right => 3,
                _ => continue,
            };

            draw_key(buffer, n, 10, 0xbc2ce4);
        }

        let mut keys_down_this_frame = [false, false, false, false];
        for key in window.get_keys_pressed(KeyRepeat::No).unwrap_or(Vec::new()) {
            keys_down_this_frame[match key {
                Key::Left => 0,
                Key::Down => 1,
                Key::Up => 2,
                Key::Right => 3,
                _ => continue,
            }] = true;
        }

        for n in notes.iter_mut() {
            n.draw(buffer, chart.bpm, time);
            if n.update_stats(&mut stats, time, &keys_down_this_frame) {
                let hitsound_src = rodio::Decoder::new(Cursor::new(hitsound_arc.clone())).unwrap();
                let _ = stream_handle.play_raw(hitsound_src.convert_samples());
            }
        }

        let _ = window.update_with_buffer(buffer, WIDTH, HEIGHT);
    }

    stats
}

fn end_screen(buffer: &mut Vec<u32>, window: &mut Window, font: &Font, chart: &Chart, stats: Stats) {
    *buffer = vec![0; WIDTH * HEIGHT];
    draw_string(buffer, font, (WIDTH / 2) as u32, (HEIGHT / 4) as u32, 0xffffff,
        &format!("{} - {}\n\naccuracy: {:.2}%\nscore: {}\nbest combo: {}", chart.title, chart.artist, stats.acc * 100.0, stats.score, stats.best_combo)
    );

    let grade = match stats.acc {
        (0.0..=0.6) => "f",
        (0.6..=0.7) => "d",
        (0.7..=0.8) => "c",
        (0.8..=0.9) => "b",
        (0.9..=0.95) => "a",
        (0.95..=0.99) => "s",
        (1.0) => "ss",
        _ => unreachable!()
    };

    let grade_tile = image::load(Cursor::new(fs::read(format!("res/grades/{}.png", grade)).unwrap()), ImageFormat::Png).unwrap();

    let mut x = WIDTH / 4;
    let mut y = HEIGHT / 4;

    for row in grade_tile.into_rgba8().enumerate_rows() {
        for pixel in row.1.enumerate() {
            buffer[(row.0 as usize + y) * WIDTH + (pixel.0 + x)] = u32::from_be_bytes(pixel.1.2.0);
        }
    }

    while !window.is_key_down(Key::Enter) && window.is_open(){
        window.update_with_buffer(buffer, WIDTH, HEIGHT);
    }

}


fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut charts = charts();
    let font = bdf::open("res/font.bdf").unwrap();

    let mut window = Window::new(
        "keys",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(Duration::from_millis(1)));

    while window.is_open() {
        println!("picking chart...");
        let chart = pick_chart(&mut buffer, &mut window, &charts, &font);
        let stats = play(&mut buffer, &mut window, &mut charts[chart], &font);
        end_screen(&mut buffer, &mut window, &font, &charts[chart], stats);

        window.update()
    }
}