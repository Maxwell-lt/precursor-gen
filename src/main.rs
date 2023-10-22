use std::{fs, path::PathBuf, collections::HashMap};

use clap::Parser;
use color_eyre::eyre::{eyre, Result};
use image::{ImageBuffer, Rgba, RgbaImage};
use serde::Deserialize;

fn main() -> Result<()> {
    // First few colors picked from a screenshot
    let line_colors = vec![
        Rgba([56u8, 85u8, 142u8, 255u8]),
        Rgba([52u8, 109u8, 227u8, 255u8]),
        Rgba([66u8, 104u8, 182u8, 255u8]),
        Rgba([88u8, 109u8, 154u8, 255u8]),
        Rgba([90u8, 135u8, 225u8, 255u8]),
        Rgba([43u8, 78u8, 148u8, 255u8]),
        Rgba([88u8, 140u8, 244u8, 255u8]),
        Rgba([78u8, 93u8, 125u8, 255u8]),
        Rgba([94u8, 139u8, 228u8, 255u8]),
        Rgba([46u8, 89u8, 176u8, 255u8]),
    ];

    let glyphs = serde_yaml::from_str::<GlyphDefs>(&fs::read_to_string("./precursor_script.yaml")?)?.glyphs;

    let args = Args::parse();
    let data = fs::read_to_string(&args.file)?;

    let (width, height) = calculate_canvas_size(&data)?;
    let mut img: RgbaImage = ImageBuffer::new(width, height);
    draw_spacers(&mut img, &line_colors, data.lines().count());
    let lines = data.lines().collect::<Vec<&str>>();
    for i in 0..lines.len() {
        draw_line(&mut img, &line_colors, lines[i], i, &glyphs)?;
    }

    let output_path = args.file.with_extension("png");
    img.save(output_path)?;

    Ok(())
}

fn calculate_canvas_size(data: &str) -> Result<(u32, u32)> {
    let lines: Vec<_> = data.lines().collect();
    let longest_line = lines
        .iter()
        .map(|s| s.len())
        .max()
        .ok_or(eyre!("finding longest line"))?;
    let min_width = longest_line * 6 * 5;
    let width_next_multiple = ((min_width + 239) / 240) * 240;

    let min_height = lines.len() * 40;
    let height_next_multiple = ((min_height + 239) / 240) * 240;
    println!("Canvas size: {}x{} blocks", width_next_multiple / 240, height_next_multiple / 240);
    Ok((width_next_multiple as u32, height_next_multiple as u32))
}

fn draw_spacers(buf: &mut RgbaImage, colors: &Vec<Rgba<u8>>, line_count: usize) {
    for i in 0..line_count {
        let color = colors[i % colors.len()];
        let start_height = i * 40;
        for j in (start_height + 2)..(start_height + 4) {
            for k in 0..(buf.width()) {
                buf.put_pixel(k, j as u32, color);
            }
        }
        for j in (start_height + 33)..(start_height + 35) {
            for k in 0..(buf.width()) {
                buf.put_pixel(k, j as u32, color);
            }
        }
    }
}

fn draw_line(buf: &mut RgbaImage, colors: &Vec<Rgba<u8>>, text: &str, line_index: usize, glyphs: &HashMap<char, [u8; 25]>) -> Result<()> {
    let color = colors[line_index % colors.len()];
    let char_count = text.chars().count();
    let space_count = text.chars().filter(|c| *c == ' ').count();
    // width of each glyph, plus the spaces between
    let min_width = 25 * char_count + (char_count - 1) * 5;
    let extra_width = buf.width() - min_width as u32;
    let extra_per_space = extra_width / space_count as u32;
    let mut spaces_with_one_more = extra_width % space_count as u32;
    let height_offset = line_index * 40 + 6;
    let mut width_offset = 0;
    for c in text.chars().rev() {
        if c.is_whitespace() {
            let mut width = 25 + extra_per_space;
            if spaces_with_one_more > 0 {
                width += 1;
                spaces_with_one_more -= 1;
            }
            draw_space(buf, color, width, (width_offset, height_offset as u32));
            width_offset += width + 5;
        } else {
            draw_char(buf, color, &c, (width_offset, height_offset as u32), glyphs)?;
            width_offset += 25 + 5;
        }
    }
    Ok(())
}

fn draw_char(buf: &mut RgbaImage, color: Rgba<u8>, c: &char, start_pos: (u32, u32), glyphs: &HashMap<char, [u8; 25]>) -> Result<()> {
    let pattern = glyphs.get(c).ok_or(eyre!("could not find glyph for char {}", c))?;
    for i in 0..pattern.len() {
        if pattern[i] == 1 {
            let width_offset = (i * 5) % 25;
            let height_offset = ((i * 5) / 25) * 5;
            let start_width = width_offset as u32 + start_pos.0;
            let start_height = height_offset as u32 + start_pos.1;
            for w in start_width..(start_width + 5) {
                for h in start_height..(start_height + 5) {
                    buf.put_pixel(w, h, color);
                }
            }
        }
    }

    Ok(())
}

fn draw_space(buf: &mut RgbaImage, color: Rgba<u8>, width: u32, start_pos: (u32, u32)) {
    let alpha_color = Rgba([color.0[0], color.0[1], color.0[2], 127u8]);
    // top line
    for w in (start_pos.0)..(start_pos.0 + width) {
        for h in (start_pos.1)..(start_pos.1 + 5) {
            buf.put_pixel(w, h, alpha_color);
        }
    }
    // bottom line
    for w in (start_pos.0)..(start_pos.0 + width) {
        for h in (start_pos.1 + 20)..(start_pos.1 + 25) {
            buf.put_pixel(w, h, alpha_color);
        }
    }
}

#[derive(Clone, Debug, Parser)]
struct Args {
    file: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
struct GlyphDefs {
    glyphs: HashMap<char, [u8; 25]>
}
