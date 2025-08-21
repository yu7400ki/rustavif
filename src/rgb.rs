//! RGB image handling and color conversion.
//!
//! This module provides functionality for working with RGB images and converting
//! between RGB and YUV color spaces. It includes support for various RGB formats,
//! chroma subsampling options, and color space conversion operations.

#![allow(non_upper_case_globals)]

use crate::{AvifError, Image, Result};
use libavif_sys::*;
use std::slice;

/// RGB pixel formats supported by AVIF.
///
/// These formats define the order and arrangement of color channels
/// in RGB pixel data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RgbFormat {
    /// 24-bit RGB format (red, green, blue)
    Rgb = avifRGBFormat_AVIF_RGB_FORMAT_RGB as isize,
    /// 32-bit RGBA format (red, green, blue, alpha)
    Rgba = avifRGBFormat_AVIF_RGB_FORMAT_RGBA as isize,
    /// 32-bit ARGB format (alpha, red, green, blue)
    Argb = avifRGBFormat_AVIF_RGB_FORMAT_ARGB as isize,
    /// 24-bit BGR format (blue, green, red)
    Bgr = avifRGBFormat_AVIF_RGB_FORMAT_BGR as isize,
    /// 32-bit BGRA format (blue, green, red, alpha)
    Bgra = avifRGBFormat_AVIF_RGB_FORMAT_BGRA as isize,
    /// 32-bit ABGR format (alpha, blue, green, red)
    Abgr = avifRGBFormat_AVIF_RGB_FORMAT_ABGR as isize,
    /// 16-bit RGB 565 format (5-bit red, 6-bit green, 5-bit blue)
    Rgb565 = avifRGBFormat_AVIF_RGB_FORMAT_RGB_565 as isize,
    /// 8-bit grayscale format
    Gray = avifRGBFormat_AVIF_RGB_FORMAT_GRAY as isize,
    /// 16-bit grayscale with alpha format
    GrayA = avifRGBFormat_AVIF_RGB_FORMAT_GRAYA as isize,
    /// 16-bit alpha with grayscale format
    AGray = avifRGBFormat_AVIF_RGB_FORMAT_AGRAY as isize,
}

impl From<RgbFormat> for avifRGBFormat {
    fn from(format: RgbFormat) -> Self {
        format as u32
    }
}

impl From<avifRGBFormat> for RgbFormat {
    fn from(format: avifRGBFormat) -> Self {
        match format {
            avifRGBFormat_AVIF_RGB_FORMAT_RGB => RgbFormat::Rgb,
            avifRGBFormat_AVIF_RGB_FORMAT_RGBA => RgbFormat::Rgba,
            avifRGBFormat_AVIF_RGB_FORMAT_ARGB => RgbFormat::Argb,
            avifRGBFormat_AVIF_RGB_FORMAT_BGR => RgbFormat::Bgr,
            avifRGBFormat_AVIF_RGB_FORMAT_BGRA => RgbFormat::Bgra,
            avifRGBFormat_AVIF_RGB_FORMAT_ABGR => RgbFormat::Abgr,
            avifRGBFormat_AVIF_RGB_FORMAT_RGB_565 => RgbFormat::Rgb565,
            avifRGBFormat_AVIF_RGB_FORMAT_GRAY => RgbFormat::Gray,
            avifRGBFormat_AVIF_RGB_FORMAT_GRAYA => RgbFormat::GrayA,
            avifRGBFormat_AVIF_RGB_FORMAT_AGRAY => RgbFormat::AGray,
            _ => RgbFormat::Rgb,
        }
    }
}

/// Chroma upsampling methods for converting YUV to RGB.
///
/// When converting from subsampled YUV formats to RGB, chroma channels
/// need to be upsampled to match the luma resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromaUpsampling {
    /// Automatically choose the best method
    Automatic = avifChromaUpsampling_AVIF_CHROMA_UPSAMPLING_AUTOMATIC as isize,
    /// Fastest upsampling (lower quality)
    Fastest = avifChromaUpsampling_AVIF_CHROMA_UPSAMPLING_FASTEST as isize,
    /// Best quality upsampling (slower)
    BestQuality = avifChromaUpsampling_AVIF_CHROMA_UPSAMPLING_BEST_QUALITY as isize,
    /// Nearest neighbor interpolation
    Nearest = avifChromaUpsampling_AVIF_CHROMA_UPSAMPLING_NEAREST as isize,
    /// Bilinear interpolation
    Bilinear = avifChromaUpsampling_AVIF_CHROMA_UPSAMPLING_BILINEAR as isize,
}

impl From<ChromaUpsampling> for avifChromaUpsampling {
    fn from(upsampling: ChromaUpsampling) -> Self {
        upsampling as u32
    }
}

/// Chroma downsampling methods for converting RGB to YUV.
///
/// When converting from RGB to subsampled YUV formats, chroma channels
/// need to be downsampled to reduce resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromaDownsampling {
    /// Automatically choose the best method
    Automatic = avifChromaDownsampling_AVIF_CHROMA_DOWNSAMPLING_AUTOMATIC as isize,
    /// Fastest downsampling (lower quality)
    Fastest = avifChromaDownsampling_AVIF_CHROMA_DOWNSAMPLING_FASTEST as isize,
    /// Best quality downsampling (slower)
    BestQuality = avifChromaDownsampling_AVIF_CHROMA_DOWNSAMPLING_BEST_QUALITY as isize,
    /// Simple averaging
    Average = avifChromaDownsampling_AVIF_CHROMA_DOWNSAMPLING_AVERAGE as isize,
    /// Sharp YUV downsampling (preserves edges better)
    SharpYuv = avifChromaDownsampling_AVIF_CHROMA_DOWNSAMPLING_SHARP_YUV as isize,
}

impl From<ChromaDownsampling> for avifChromaDownsampling {
    fn from(downsampling: ChromaDownsampling) -> Self {
        downsampling as u32
    }
}

/// RGB image structure for color space conversion and pixel manipulation.
///
/// This structure provides a safe Rust interface to libavif's RGB image
/// handling, including conversion to/from YUV color spaces and pixel format
/// transformations.
///
/// # Lifetime
/// The lifetime parameter `'a` ensures that the pixel data remains valid
/// for the duration of the RgbImage's existence.
pub struct RgbImage<'a> {
    pub(crate) inner: avifRGBImage,
    _marker: std::marker::PhantomData<&'a [u8]>,
}

impl<'a> RgbImage<'a> {
    /// Creates an RGB image from existing pixel data.
    ///
    /// # Arguments
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels  
    /// * `depth` - Bit depth of the image
    /// * `format` - RGB pixel format
    /// * `pixels` - Mutable slice containing pixel data
    ///
    /// # Returns
    /// A new RgbImage instance or an error if the pixel data is insufficient.
    pub fn from_pixels(
        width: u32,
        height: u32,
        depth: crate::BitDepth,
        format: RgbFormat,
        pixels: &'a mut [u8],
    ) -> Result<Self> {
        let pixel_size = unsafe { avifRGBFormatChannelCount(format.into()) };
        let expected_row_bytes = width * pixel_size;
        let expected_size = (expected_row_bytes * height) as usize;

        if pixels.len() < expected_size {
            return Err(AvifError::InvalidArgument);
        }

        Ok(Self {
            inner: avifRGBImage {
                width,
                height,
                depth: depth.into(),
                format: format.into(),
                chromaUpsampling: ChromaUpsampling::Automatic.into(),
                chromaDownsampling: ChromaDownsampling::Automatic.into(),
                avoidLibYUV: 0,
                ignoreAlpha: 0,
                alphaPremultiplied: 0,
                isFloat: 0,
                maxThreads: 1,
                pixels: pixels.as_mut_ptr(),
                rowBytes: expected_row_bytes,
            },
            _marker: std::marker::PhantomData,
        })
    }

    /// Returns the image width in pixels.
    pub fn width(&self) -> u32 {
        self.inner.width
    }

    /// Returns the image height in pixels.
    pub fn height(&self) -> u32 {
        self.inner.height
    }

    /// Returns the bit depth of the image.
    pub fn depth(&self) -> crate::BitDepth {
        crate::BitDepth::try_from(self.inner.depth).unwrap_or(crate::BitDepth::Eight)
    }

    /// Returns the RGB pixel format.
    pub fn format(&self) -> RgbFormat {
        self.inner.format.into()
    }

    /// Sets the RGB pixel format.
    pub fn set_format(&mut self, format: RgbFormat) {
        self.inner.format = format.into();
    }

    /// Sets the chroma upsampling method.
    pub fn set_chroma_upsampling(&mut self, upsampling: ChromaUpsampling) {
        self.inner.chromaUpsampling = upsampling.into();
    }

    /// Sets the chroma downsampling method.
    pub fn set_chroma_downsampling(&mut self, downsampling: ChromaDownsampling) {
        self.inner.chromaDownsampling = downsampling.into();
    }

    /// Sets whether to avoid using libyuv for color conversion.
    pub fn set_avoid_libyuv(&mut self, avoid: bool) {
        self.inner.avoidLibYUV = if avoid { 1 } else { 0 };
    }

    /// Sets whether to ignore the alpha channel.
    pub fn set_ignore_alpha(&mut self, ignore: bool) {
        self.inner.ignoreAlpha = if ignore { 1 } else { 0 };
    }

    /// Sets whether alpha is premultiplied.
    pub fn set_alpha_premultiplied(&mut self, premultiplied: bool) {
        self.inner.alphaPremultiplied = if premultiplied { 1 } else { 0 };
    }

    /// Sets whether the pixel data is floating point.
    pub fn set_is_float(&mut self, is_float: bool) {
        self.inner.isFloat = if is_float { 1 } else { 0 };
    }

    /// Sets the maximum number of threads to use for conversion.
    pub fn set_max_threads(&mut self, threads: u32) {
        self.inner.maxThreads = threads.min(1024) as i32;
    }

    /// Returns the size of a single pixel in bytes.
    pub fn pixel_size(&self) -> u32 {
        unsafe { avifRGBImagePixelSize(&self.inner) }
    }

    /// Returns the number of channels per pixel.
    pub fn channel_count(&self) -> u32 {
        unsafe { avifRGBFormatChannelCount(self.inner.format) }
    }

    /// Returns true if the format includes an alpha channel.
    pub fn has_alpha(&self) -> bool {
        unsafe { avifRGBFormatHasAlpha(self.inner.format) != 0 }
    }

    /// Returns true if the format is grayscale.
    pub fn is_gray(&self) -> bool {
        unsafe { avifRGBFormatIsGray(self.inner.format) != 0 }
    }

    /// Returns the pixel data as a byte slice.
    pub fn pixels(&self) -> &[u8] {
        let size = (self.inner.rowBytes * self.inner.height) as usize;
        unsafe { slice::from_raw_parts(self.inner.pixels, size) }
    }

    /// Returns the pixel data as a mutable byte slice.
    pub fn pixels_mut(&mut self) -> &mut [u8] {
        let size = (self.inner.rowBytes * self.inner.height) as usize;
        unsafe { slice::from_raw_parts_mut(self.inner.pixels, size) }
    }

    /// Returns the number of bytes per row.
    pub fn row_bytes(&self) -> u32 {
        self.inner.rowBytes
    }

    /// Converts this RGB image to a YUV image.
    ///
    /// # Arguments
    /// * `yuv_format` - The target YUV pixel format
    ///
    /// # Returns
    /// A new YUV Image or an error if conversion fails.
    pub fn to_yuv_image(&self, yuv_format: crate::PixelFormat) -> Result<Image> {
        let mut yuv_image = Image::new(self.width(), self.height(), self.depth(), yuv_format)?;
        yuv_image.allocate_planes()?;

        let result = unsafe { avifImageRGBToYUV(yuv_image.inner, &self.inner) };
        if result != avifResult_AVIF_RESULT_OK {
            Err(AvifError::from(result))
        } else {
            Ok(yuv_image)
        }
    }

    /// Premultiplies the alpha channel with the color channels.
    ///
    /// This operation multiplies each color channel by the alpha value,
    /// which is useful for certain compositing operations.
    pub fn premultiply_alpha(&mut self) -> Result<()> {
        let result = unsafe { avifRGBImagePremultiplyAlpha(&mut self.inner) };
        if result != avifResult_AVIF_RESULT_OK {
            Err(AvifError::from(result))
        } else {
            Ok(())
        }
    }

    /// Unpremultiplies the alpha channel from the color channels.
    ///
    /// This reverses the premultiplication operation, dividing each
    /// color channel by the alpha value.
    pub fn unpremultiply_alpha(&mut self) -> Result<()> {
        let result = unsafe { avifRGBImageUnpremultiplyAlpha(&mut self.inner) };
        if result != avifResult_AVIF_RESULT_OK {
            Err(AvifError::from(result))
        } else {
            Ok(())
        }
    }
}
