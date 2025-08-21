use rustavif::{BitDepth, Encoder, PixelFormat, Result, RgbFormat, RgbImage};
use std::fs::File;
use std::io::Write;

fn main() -> Result<()> {
    // Animation parameters
    let width = 64u32;
    let height = 64u32;
    let depth = BitDepth::Eight;
    let format = RgbFormat::Rgba;
    let num_frames = 10;

    // Create encoder for animation
    let mut encoder = Encoder::new()?;
    encoder.set_quality(70);
    encoder.set_speed(6);
    encoder.set_timescale(30); // 30 FPS
    encoder.set_repetition_count(0); // Infinite loop

    println!("Creating animated AVIF with {} frames...", num_frames);

    // Generate and add frames
    for frame in 0..num_frames {
        println!("Processing frame {}/{}", frame + 1, num_frames);

        // Create RGB buffer for this frame
        let mut rgb_buffer = vec![0u8; (width * height * 4) as usize]; // RGBA

        // Create animated pattern (rotating colors)
        let phase = (frame as f32 / num_frames as f32) * 2.0 * std::f32::consts::PI;

        for y in 0..height {
            for x in 0..width {
                let index = ((y * width + x) * 4) as usize;

                // Create circular wave pattern
                let center_x = width as f32 / 2.0;
                let center_y = height as f32 / 2.0;
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                let wave = (distance * 0.2 + phase).sin() * 0.5 + 0.5;

                // Animate colors
                let red = ((x as f32 / width as f32) * wave * 255.0) as u8;
                let green = ((y as f32 / height as f32) * wave * 255.0) as u8;
                let blue = ((1.0 - wave) * 255.0) as u8;

                rgb_buffer[index] = red; // R
                rgb_buffer[index + 1] = green; // G  
                rgb_buffer[index + 2] = blue; // B
                rgb_buffer[index + 3] = 255; // A
            }
        }

        // Create RgbImage from buffer
        let rgb_image = RgbImage::from_pixels(width, height, depth, format, &mut rgb_buffer)?;

        // Convert RGB to YUV
        let yuv_image = rgb_image.to_yuv_image(PixelFormat::Yuv420)?;

        // Add frame to animation (duration: 1 second / 30 FPS = 1/30 second)
        let duration_in_timescales = 1; // 1/30 second at 30 FPS
        encoder.add_image(&yuv_image, duration_in_timescales, Default::default())?;
    }

    // Finish encoding animation
    println!("Finalizing animation...");
    let output = encoder.finish()?;

    // Save to file
    let filename = "animation.avif";
    let mut file = File::create(filename).map_err(|_| rustavif::AvifError::IoError)?;
    file.write_all(output.as_slice())
        .map_err(|_| rustavif::AvifError::IoError)?;

    println!("✓ Successfully created animated AVIF!");
    println!("File: {}", filename);
    println!("Size: {} bytes", output.as_slice().len());
    println!("Frames: {}", num_frames);
    println!("Resolution: {}x{}", width, height);
    println!("Bit depth: {:?}", depth);
    println!(
        "Duration: ~{:.1} seconds at 30 FPS",
        num_frames as f32 / 30.0
    );

    Ok(())
}
