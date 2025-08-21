//! Rust bindings for libavif - AVIF image encoding and decoding library.
//!
//! This crate provides safe Rust bindings for the libavif C library, allowing you to encode and
//! decode AVIF images. AVIF is a modern image format based on the AV1 video codec, offering
//! superior compression efficiency compared to traditional formats like JPEG and PNG.

#![allow(non_upper_case_globals)]

use libavif_sys::*;
use std::ptr::null_mut;

pub mod encoder;
pub mod error;
pub mod rgb;

pub use encoder::Encoder;
pub use error::AvifError;
pub use rgb::{ChromaDownsampling, ChromaUpsampling, RgbFormat, RgbImage};

/// A convenience type alias for Results with AvifError.
pub type Result<T> = std::result::Result<T, AvifError>;

/// Supported bit depths for AVIF images.
///
/// Different bit depths allow for varying levels of color precision:
/// - 8-bit: Standard precision, widely supported
/// - 10-bit: Higher precision, better for HDR content
/// - 12-bit: Maximum precision, professional use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitDepth {
    /// 8-bit depth (standard precision)
    Eight = 8,
    /// 10-bit depth (higher precision)
    Ten = 10,
    /// 12-bit depth (maximum precision)
    Twelve = 12,
}

impl From<BitDepth> for u32 {
    fn from(depth: BitDepth) -> Self {
        depth as u32
    }
}

impl TryFrom<u32> for BitDepth {
    type Error = AvifError;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            8 => Ok(BitDepth::Eight),
            10 => Ok(BitDepth::Ten),
            12 => Ok(BitDepth::Twelve),
            _ => Err(AvifError::InvalidArgument),
        }
    }
}

/// YUV pixel format for AVIF images.
///
/// Different YUV formats provide different levels of compression and quality:
/// - YUV444: Full chroma resolution (no chroma subsampling)
/// - YUV422: Half chroma width resolution
/// - YUV420: Half chroma width and height resolution (most common)
/// - YUV400: Grayscale (no chroma information)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// No format specified
    None = avifPixelFormat_AVIF_PIXEL_FORMAT_NONE as isize,
    /// Full chroma resolution (4:4:4)
    Yuv444 = avifPixelFormat_AVIF_PIXEL_FORMAT_YUV444 as isize,
    /// Half chroma width (4:2:2)
    Yuv422 = avifPixelFormat_AVIF_PIXEL_FORMAT_YUV422 as isize,
    /// Half chroma width and height (4:2:0) - most common
    Yuv420 = avifPixelFormat_AVIF_PIXEL_FORMAT_YUV420 as isize,
    /// Grayscale (4:0:0)
    Yuv400 = avifPixelFormat_AVIF_PIXEL_FORMAT_YUV400 as isize,
}

impl From<PixelFormat> for avifPixelFormat {
    fn from(format: PixelFormat) -> Self {
        format as _
    }
}

impl From<avifPixelFormat> for PixelFormat {
    fn from(format: avifPixelFormat) -> Self {
        match format {
            avifPixelFormat_AVIF_PIXEL_FORMAT_YUV444 => PixelFormat::Yuv444,
            avifPixelFormat_AVIF_PIXEL_FORMAT_YUV422 => PixelFormat::Yuv422,
            avifPixelFormat_AVIF_PIXEL_FORMAT_YUV420 => PixelFormat::Yuv420,
            avifPixelFormat_AVIF_PIXEL_FORMAT_YUV400 => PixelFormat::Yuv400,
            _ => PixelFormat::None,
        }
    }
}

/// A wrapper around libavif's RWData structure for managing read/write data buffers.
///
/// This structure automatically manages the memory lifecycle of data buffers
/// used for encoding and decoding AVIF images.
pub struct RwData {
    pub(crate) inner: avifRWData,
}

impl RwData {
    /// Creates a new empty RwData buffer.
    pub fn new() -> Self {
        Self {
            inner: avifRWData {
                data: null_mut(),
                size: 0,
            },
        }
    }

    /// Returns the data as a byte slice.
    ///
    /// This provides safe access to the underlying buffer data without
    /// transferring ownership.
    pub fn as_slice(&self) -> &[u8] {
        if self.inner.data.is_null() || self.inner.size == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.inner.data, self.inner.size) }
        }
    }
}

impl Drop for RwData {
    fn drop(&mut self) {
        unsafe {
            avifRWDataFree(&mut self.inner);
        }
    }
}

impl Default for RwData {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents an AVIF image with YUV pixel data.
///
/// This structure wraps libavif's avifImage and provides safe Rust methods
/// for image manipulation, including creation, copying, scaling, and format conversion.
pub struct Image {
    pub(crate) inner: *mut avifImage,
}

impl Image {
    /// Creates a new AVIF image with the specified dimensions and format.
    ///
    /// # Arguments
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    /// * `depth` - Bit depth (8, 10, or 12 bits)
    /// * `yuv_format` - YUV pixel format
    ///
    /// # Returns
    /// A new Image instance or an error if creation fails.
    pub fn new(width: u32, height: u32, depth: BitDepth, yuv_format: PixelFormat) -> Result<Self> {
        let inner = unsafe { avifImageCreate(width, height, depth.into(), yuv_format.into()) };
        if inner.is_null() {
            Err(AvifError::OutOfMemory)
        } else {
            Ok(Self { inner })
        }
    }

    /// Returns the image width in pixels.
    pub fn width(&self) -> u32 {
        unsafe { (*self.inner).width as u32 }
    }

    /// Returns the image height in pixels.
    pub fn height(&self) -> u32 {
        unsafe { (*self.inner).height as u32 }
    }

    /// Returns the bit depth of the image.
    pub fn depth(&self) -> BitDepth {
        let depth_value = unsafe { (*self.inner).depth };
        BitDepth::try_from(depth_value).unwrap_or(BitDepth::Eight)
    }

    /// Returns the YUV pixel format.
    pub fn yuv_format(&self) -> PixelFormat {
        unsafe { (*self.inner).yuvFormat.into() }
    }

    /// Sets the YUV pixel format.
    pub fn set_yuv_format(&mut self, format: PixelFormat) {
        unsafe { (*self.inner).yuvFormat = format.into() };
    }

    /// Returns whether alpha is premultiplied.
    pub fn alpha_premultiplied(&self) -> bool {
        unsafe { (*self.inner).alphaPremultiplied != 0 }
    }

    /// Sets whether alpha is premultiplied.
    pub fn set_alpha_premultiplied(&mut self, premultiplied: bool) {
        unsafe { (*self.inner).alphaPremultiplied = if premultiplied { 1 } else { 0 } };
    }

    /// Returns the YUV range (full or limited).
    pub fn yuv_range(&self) -> avifRange {
        unsafe { (*self.inner).yuvRange }
    }

    /// Sets the YUV range (full or limited).
    pub fn set_yuv_range(&mut self, range: avifRange) {
        unsafe { (*self.inner).yuvRange = range };
    }

    /// Returns the color primaries.
    pub fn color_primaries(&self) -> avifColorPrimaries {
        unsafe { (*self.inner).colorPrimaries }
    }

    /// Sets the color primaries.
    pub fn set_color_primaries(&mut self, primaries: avifColorPrimaries) {
        unsafe { (*self.inner).colorPrimaries = primaries };
    }

    /// Returns the transfer characteristics.
    pub fn transfer_characteristics(&self) -> avifTransferCharacteristics {
        unsafe { (*self.inner).transferCharacteristics }
    }

    /// Sets the transfer characteristics.
    pub fn set_transfer_characteristics(&mut self, tc: avifTransferCharacteristics) {
        unsafe { (*self.inner).transferCharacteristics = tc };
    }

    /// Returns the matrix coefficients.
    pub fn matrix_coefficients(&self) -> avifMatrixCoefficients {
        unsafe { (*self.inner).matrixCoefficients }
    }

    /// Sets the matrix coefficients.
    pub fn set_matrix_coefficients(&mut self, mc: avifMatrixCoefficients) {
        unsafe { (*self.inner).matrixCoefficients = mc };
    }

    /// Allocates memory for the image planes (YUV and alpha).
    ///
    /// This must be called before writing pixel data to the image.
    pub fn allocate_planes(&mut self) -> Result<()> {
        let result = unsafe {
            avifImageAllocatePlanes(
                self.inner,
                avifPlanesFlag_AVIF_PLANES_ALL as avifPlanesFlags,
            )
        };
        if result != avifResult_AVIF_RESULT_OK {
            Err(AvifError::from(result))
        } else {
            Ok(())
        }
    }

    /// Frees the memory used by the image planes.
    pub fn free_planes(&mut self) {
        unsafe {
            avifImageFreePlanes(
                self.inner,
                avifPlanesFlag_AVIF_PLANES_ALL as avifPlanesFlags,
            )
        };
    }

    /// Transfers ownership of planes from this image to another image.
    ///
    /// After this operation, this image will have empty planes.
    pub fn steal_planes(&mut self, to_image: &mut Self) {
        unsafe {
            avifImageStealPlanes(
                to_image.inner,
                self.inner,
                avifPlanesFlag_AVIF_PLANES_ALL as avifPlanesFlags,
            )
        };
    }

    /// Creates a copy of this image.
    ///
    /// This creates a new image with the same dimensions and copies all pixel data.
    pub fn copy(&self) -> Result<Self> {
        let copy = Self::new(self.width(), self.height(), self.depth(), self.yuv_format())?;
        let result = unsafe {
            avifImageCopy(
                copy.inner,
                self.inner,
                avifPlanesFlag_AVIF_PLANES_ALL as avifPlanesFlags,
            )
        };
        if result != avifResult_AVIF_RESULT_OK {
            Err(AvifError::from(result))
        } else {
            Ok(copy)
        }
    }

    /// Scales the image to new dimensions.
    ///
    /// # Arguments
    /// * `new_width` - Target width in pixels
    /// * `new_height` - Target height in pixels
    pub fn scale(&mut self, new_width: u32, new_height: u32) -> Result<()> {
        let result = unsafe { avifImageScale(self.inner, new_width, new_height, null_mut()) };
        if result != avifResult_AVIF_RESULT_OK {
            Err(AvifError::from(result))
        } else {
            Ok(())
        }
    }

    /// Returns true if the image has no alpha channel or if all alpha values are fully opaque.
    pub fn is_opaque(&self) -> bool {
        unsafe { avifImageIsOpaque(self.inner) != 0 }
    }

    /// Returns true if the image uses 16-bit integer storage internally.
    pub fn uses_u16(&self) -> bool {
        unsafe { avifImageUsesU16(self.inner) != 0 }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                avifImageDestroy(self.inner);
            }
        }
    }
}
