//! AVIF encoder functionality.
//!
//! This module provides the `Encoder` struct and related types for encoding images
//! to the AVIF format. The encoder supports various configuration options including
//! quality settings, codec selection, tiling, and animation support.

use crate::{AvifError, Image, Result, RwData};
use libavif_sys::*;
use std::{ffi::CString, ops};

/// Available codec choices for AVIF encoding.
///
/// Different codecs may have different performance characteristics,
/// quality profiles, and feature support.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoderCodecChoice {
    /// Automatically select the best available codec
    Auto = avifCodecChoice_AVIF_CODEC_CHOICE_AUTO as isize,
    /// Use the AOM AV1 encoder (reference implementation)
    #[cfg(feature = "codec-aom")]
    Aom = avifCodecChoice_AVIF_CODEC_CHOICE_AOM as isize,
    /// Use the Rav1e encoder (Rust implementation)
    #[cfg(feature = "codec-rav1e")]
    Rav1e = avifCodecChoice_AVIF_CODEC_CHOICE_RAV1E as isize,
    /// Use the SVT-AV1 encoder (optimized for speed)
    #[cfg(feature = "codec-svt")]
    Svt = avifCodecChoice_AVIF_CODEC_CHOICE_SVT as isize,
}

impl From<EncoderCodecChoice> for avifCodecChoice {
    fn from(choice: EncoderCodecChoice) -> Self {
        choice as _
    }
}

/// Flags for controlling how images are added to the encoder.
///
/// These flags can be combined using the bitwise OR operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddImageFlags(u32);

impl AddImageFlags {
    /// No special flags
    pub const NONE: Self = Self(avifAddImageFlag_AVIF_ADD_IMAGE_FLAG_NONE as u32);
    /// Force this image to be a keyframe
    pub const FORCE_KEYFRAME: Self =
        Self(avifAddImageFlag_AVIF_ADD_IMAGE_FLAG_FORCE_KEYFRAME as u32);
    /// This is the only image (single-image AVIF)
    pub const SINGLE: Self = Self(avifAddImageFlag_AVIF_ADD_IMAGE_FLAG_SINGLE as u32);
}

impl AddImageFlags {
    /// Returns the raw flag bits.
    pub fn bits(&self) -> u32 {
        self.0
    }
}

impl Default for AddImageFlags {
    fn default() -> Self {
        Self::NONE
    }
}

impl ops::BitOr for AddImageFlags {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        Self(self.0 | other.0)
    }
}

/// AVIF encoder for converting images to AVIF format.
///
/// The encoder provides various configuration options for controlling
/// the encoding process, including quality settings, codec selection,
/// and animation support.
pub struct Encoder {
    inner: *mut avifEncoder,
}

impl Encoder {
    /// Creates a new AVIF encoder.
    ///
    /// # Returns
    /// A new encoder instance or an error if creation fails.
    pub fn new() -> Result<Self> {
        let inner = unsafe { avifEncoderCreate() };
        if inner.is_null() {
            Err(AvifError::OutOfMemory)
        } else {
            Ok(Self { inner })
        }
    }

    /// Sets the codec choice for encoding.
    ///
    /// # Arguments
    /// * `choice` - The codec to use for encoding
    pub fn set_codec_choice(&mut self, choice: EncoderCodecChoice) {
        unsafe {
            (*self.inner).codecChoice = choice.into();
        }
    }

    /// Sets the maximum number of threads to use for encoding.
    ///
    /// # Arguments
    /// * `threads` - Number of threads (clamped to 1024)
    pub fn set_max_threads(&mut self, threads: u32) {
        unsafe {
            (*self.inner).maxThreads = threads.min(1024) as i32;
        }
    }

    /// Sets the encoding speed/quality tradeoff.
    ///
    /// # Arguments
    /// * `speed` - Speed setting from 0 (slowest, best quality) to 10 (fastest, lower quality)
    pub fn set_speed(&mut self, speed: u8) {
        unsafe {
            (*self.inner).speed = speed.min(10) as i32;
        }
    }

    /// Sets the keyframe interval for animations.
    ///
    /// # Arguments
    /// * `interval` - Number of frames between keyframes (0 = all keyframes)
    pub fn set_keyframe_interval(&mut self, interval: u32) {
        unsafe {
            (*self.inner).keyframeInterval = interval as i32;
        }
    }

    /// Sets the timescale for animations.
    ///
    /// # Arguments
    /// * `timescale` - Time units per second (e.g., 1000 for milliseconds)
    pub fn set_timescale(&mut self, timescale: u64) {
        unsafe {
            (*self.inner).timescale = timescale;
        }
    }

    /// Sets the repetition count for animations.
    ///
    /// # Arguments
    /// * `count` - Number of repetitions (0 = infinite loop)
    pub fn set_repetition_count(&mut self, count: u32) {
        unsafe {
            (*self.inner).repetitionCount = (count as i32) - 1;
        }
    }

    /// Sets the quality for color channels.
    ///
    /// # Arguments
    /// * `quality` - Quality from 0 (lowest) to 100 (highest/lossless)
    pub fn set_quality(&mut self, quality: u8) {
        unsafe {
            (*self.inner).quality = quality.min(100) as i32;
        }
    }

    /// Sets the quality for the alpha channel.
    ///
    /// # Arguments
    /// * `quality` - Quality from 0 (lowest) to 100 (highest/lossless)
    pub fn set_quality_alpha(&mut self, quality: u8) {
        unsafe {
            (*self.inner).qualityAlpha = quality.min(100) as i32;
        }
    }

    /// Sets the quantizer range for color channels.
    ///
    /// # Arguments
    /// * `min` - Minimum quantizer value (0-100)
    /// * `max` - Maximum quantizer value (0-100)
    pub fn set_quantizer_range(&mut self, min: u8, max: u8) {
        unsafe {
            (*self.inner).minQuantizer = min.min(100) as i32;
            (*self.inner).maxQuantizer = max.min(100) as i32;
        }
    }

    /// Sets the quantizer range for the alpha channel.
    ///
    /// # Arguments
    /// * `min` - Minimum quantizer value (0-100)
    /// * `max` - Maximum quantizer value (0-100)
    pub fn set_quantizer_alpha_range(&mut self, min: u8, max: u8) {
        unsafe {
            (*self.inner).minQuantizerAlpha = min.min(100) as i32;
            (*self.inner).maxQuantizerAlpha = max.min(100) as i32;
        }
    }

    /// Sets the tiling configuration.
    ///
    /// Tiling can improve encoding performance and enable parallel decoding.
    ///
    /// # Arguments
    /// * `tile_rows_log2` - Log2 of the number of tile rows (0-6)
    /// * `tile_cols_log2` - Log2 of the number of tile columns (0-6)
    pub fn set_tiling(&mut self, tile_rows_log2: u8, tile_cols_log2: u8) {
        unsafe {
            (*self.inner).tileRowsLog2 = tile_rows_log2.min(6) as i32;
            (*self.inner).tileColsLog2 = tile_cols_log2.min(6) as i32;
        }
    }

    /// Enables or disables automatic tiling.
    ///
    /// When enabled, the encoder will automatically determine optimal tiling.
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable automatic tiling
    pub fn set_auto_tiling(&mut self, enabled: bool) {
        unsafe {
            (*self.inner).autoTiling = if enabled { 1 } else { 0 };
        }
    }

    /// Adds an image to the encoder for animation sequences.
    ///
    /// # Arguments
    /// * `image` - The image to add
    /// * `duration_in_timescales` - Duration this image should be displayed
    /// * `add_image_flags` - Flags controlling how the image is added
    ///
    /// # Returns
    /// Ok(()) on success, or an error if the operation fails.
    pub fn add_image(
        &mut self,
        image: &Image,
        duration_in_timescales: u64,
        add_image_flags: AddImageFlags,
    ) -> Result<()> {
        let result = unsafe {
            avifEncoderAddImage(
                self.inner,
                image.inner,
                duration_in_timescales,
                add_image_flags.bits(),
            )
        };
        if result != avifResult_AVIF_RESULT_OK {
            Err(AvifError::from(result))
        } else {
            Ok(())
        }
    }

    /// Adds a grid of images as a single tiled image.
    ///
    /// This creates a single AVIF image composed of multiple smaller images
    /// arranged in a grid pattern.
    ///
    /// # Arguments
    /// * `grid_cols` - Number of columns in the grid
    /// * `grid_rows` - Number of rows in the grid  
    /// * `images` - Array of images to arrange in the grid
    /// * `add_image_flags` - Flags controlling how the grid is added
    ///
    /// # Returns
    /// Ok(()) on success, or an error if the operation fails.
    pub fn add_image_grid(
        &mut self,
        grid_cols: u32,
        grid_rows: u32,
        images: &[&Image],
        add_image_flags: AddImageFlags,
    ) -> Result<()> {
        let image_ptrs: Vec<*const avifImage> =
            images.iter().map(|img| img.inner as *const _).collect();
        let result = unsafe {
            avifEncoderAddImageGrid(
                self.inner,
                grid_cols,
                grid_rows,
                image_ptrs.as_ptr(),
                add_image_flags.bits(),
            )
        };
        if result != avifResult_AVIF_RESULT_OK {
            Err(AvifError::from(result))
        } else {
            Ok(())
        }
    }

    /// Finalizes encoding and returns the AVIF data for animation sequences.
    ///
    /// This should be called after all images have been added via `add_image()`
    /// or `add_image_grid()` calls.
    ///
    /// # Returns
    /// The encoded AVIF data, or an error if encoding fails.
    pub fn finish(&mut self) -> Result<RwData> {
        let mut output = RwData::new();
        let result = unsafe { avifEncoderFinish(self.inner, &mut output.inner) };
        if result != avifResult_AVIF_RESULT_OK {
            Err(AvifError::from(result))
        } else {
            Ok(output)
        }
    }

    /// Encodes a single image to AVIF format.
    ///
    /// This is a convenience method for encoding a single image without
    /// using the animation workflow.
    ///
    /// # Arguments
    /// * `image` - The image to encode
    ///
    /// # Returns
    /// The encoded AVIF data, or an error if encoding fails.
    pub fn write(&mut self, image: &Image) -> Result<RwData> {
        let mut output = RwData::new();
        let result = unsafe { avifEncoderWrite(self.inner, image.inner, &mut output.inner) };
        if result != avifResult_AVIF_RESULT_OK {
            Err(AvifError::from(result))
        } else {
            Ok(output)
        }
    }

    /// Sets a codec-specific option.
    ///
    /// These options are passed directly to the underlying codec and
    /// can be used to fine-tune encoding behavior.
    ///
    /// # Arguments
    /// * `key` - The option name
    /// * `value` - The option value
    ///
    /// # Returns
    /// Ok(()) on success, or an error if the option is invalid.
    pub fn set_codec_specific_option(&mut self, key: &str, value: &str) -> Result<()> {
        let key_c = CString::new(key).map_err(|_| AvifError::InvalidArgument)?;
        let value_c = CString::new(value).map_err(|_| AvifError::InvalidArgument)?;

        let result = unsafe {
            avifEncoderSetCodecSpecificOption(self.inner, key_c.as_ptr(), value_c.as_ptr())
        };

        if result != avifResult_AVIF_RESULT_OK {
            Err(AvifError::from(result))
        } else {
            Ok(())
        }
    }

    /// Returns the size of the gain map in bytes.
    ///
    /// Gain maps are used for HDR image support.
    pub fn get_gain_map_size_bytes(&self) -> usize {
        unsafe { avifEncoderGetGainMapSizeBytes(self.inner) }
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                avifEncoderDestroy(self.inner);
            }
        }
    }
}

impl Default for Encoder {
    fn default() -> Self {
        Self::new().expect("Failed to create encoder")
    }
}
