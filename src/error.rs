//! Error types for AVIF operations.
//!
//! This module defines the `AvifError` enum which represents all possible
//! error conditions that can occur during AVIF encoding and decoding operations.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use libavif_sys::*;
use std::fmt;

/// Error types that can occur during AVIF operations.
///
/// This enum represents all possible error conditions that can arise
/// when encoding or decoding AVIF images using libavif.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvifError {
    /// An unknown error occurred
    UnknownError,
    /// Invalid file type (not a valid AVIF file)
    InvalidFtyp,
    /// No content found in the file
    NoContent,
    /// No YUV format was selected for the operation
    NoYuvFormatSelected,
    /// Color format conversion failed
    ReformatFailed,
    /// Bit depth is not supported
    UnsupportedDepth,
    /// Encoding of color channels failed
    EncodeColorFailed,
    /// Encoding of alpha channel failed
    EncodeAlphaFailed,
    /// Base Media File Format (BMFF) parsing failed
    BmffParseFailed,
    /// Required image item is missing from the file
    MissingImageItem,
    /// Decoding of color channels failed
    DecodeColorFailed,
    /// Decoding of alpha channel failed
    DecodeAlphaFailed,
    /// Color and alpha channel dimensions don't match
    ColorAlphaSizeMismatch,
    /// Image Spatial Extents (ISPE) size mismatch
    IspeSizeMismatch,
    /// No suitable codec is available
    NoCodecAvailable,
    /// No more images remaining in sequence
    NoImagesRemaining,
    /// EXIF metadata payload is invalid
    InvalidExifPayload,
    /// Image grid configuration is invalid
    InvalidImageGrid,
    /// Codec-specific option is invalid
    InvalidCodecSpecificOption,
    /// Input data is truncated or incomplete
    TruncatedData,
    /// IO handler is not set
    IoNotSet,
    /// IO operation failed
    IoError,
    /// Waiting for IO operation to complete
    WaitingOnIo,
    /// Invalid argument provided to function
    InvalidArgument,
    /// Feature is not implemented
    NotImplemented,
    /// Out of memory
    OutOfMemory,
    /// Image format is incompatible with operation
    IncompatibleImage,
    /// Gain map encoding failed
    EncodeGainMapFailed,
    /// Gain map decoding failed
    DecodeGainMapFailed,
    /// Tone mapped image is invalid
    InvalidToneMappedImage,
    /// Unknown error type with raw code
    UnknownType(u32),
}

impl From<avifResult> for AvifError {
    fn from(result: avifResult) -> Self {
        match result {
            avifResult_AVIF_RESULT_OK => {
                unreachable!("avifResult_AVIF_RESULT_OK should not be converted to AvifError")
            }
            avifResult_AVIF_RESULT_UNKNOWN_ERROR => AvifError::UnknownError,
            avifResult_AVIF_RESULT_INVALID_FTYP => AvifError::InvalidFtyp,
            avifResult_AVIF_RESULT_NO_CONTENT => AvifError::NoContent,
            avifResult_AVIF_RESULT_NO_YUV_FORMAT_SELECTED => AvifError::NoYuvFormatSelected,
            avifResult_AVIF_RESULT_REFORMAT_FAILED => AvifError::ReformatFailed,
            avifResult_AVIF_RESULT_UNSUPPORTED_DEPTH => AvifError::UnsupportedDepth,
            avifResult_AVIF_RESULT_ENCODE_COLOR_FAILED => AvifError::EncodeColorFailed,
            avifResult_AVIF_RESULT_ENCODE_ALPHA_FAILED => AvifError::EncodeAlphaFailed,
            avifResult_AVIF_RESULT_BMFF_PARSE_FAILED => AvifError::BmffParseFailed,
            avifResult_AVIF_RESULT_MISSING_IMAGE_ITEM => AvifError::MissingImageItem,
            avifResult_AVIF_RESULT_DECODE_COLOR_FAILED => AvifError::DecodeColorFailed,
            avifResult_AVIF_RESULT_DECODE_ALPHA_FAILED => AvifError::DecodeAlphaFailed,
            avifResult_AVIF_RESULT_COLOR_ALPHA_SIZE_MISMATCH => AvifError::ColorAlphaSizeMismatch,
            avifResult_AVIF_RESULT_ISPE_SIZE_MISMATCH => AvifError::IspeSizeMismatch,
            avifResult_AVIF_RESULT_NO_CODEC_AVAILABLE => AvifError::NoCodecAvailable,
            avifResult_AVIF_RESULT_NO_IMAGES_REMAINING => AvifError::NoImagesRemaining,
            avifResult_AVIF_RESULT_INVALID_EXIF_PAYLOAD => AvifError::InvalidExifPayload,
            avifResult_AVIF_RESULT_INVALID_IMAGE_GRID => AvifError::InvalidImageGrid,
            avifResult_AVIF_RESULT_INVALID_CODEC_SPECIFIC_OPTION => {
                AvifError::InvalidCodecSpecificOption
            }
            avifResult_AVIF_RESULT_TRUNCATED_DATA => AvifError::TruncatedData,
            avifResult_AVIF_RESULT_IO_NOT_SET => AvifError::IoNotSet,
            avifResult_AVIF_RESULT_IO_ERROR => AvifError::IoError,
            avifResult_AVIF_RESULT_WAITING_ON_IO => AvifError::WaitingOnIo,
            avifResult_AVIF_RESULT_INVALID_ARGUMENT => AvifError::InvalidArgument,
            avifResult_AVIF_RESULT_NOT_IMPLEMENTED => AvifError::NotImplemented,
            avifResult_AVIF_RESULT_OUT_OF_MEMORY => AvifError::OutOfMemory,
            avifResult_AVIF_RESULT_INCOMPATIBLE_IMAGE => AvifError::IncompatibleImage,
            avifResult_AVIF_RESULT_ENCODE_GAIN_MAP_FAILED => AvifError::EncodeGainMapFailed,
            avifResult_AVIF_RESULT_DECODE_GAIN_MAP_FAILED => AvifError::DecodeGainMapFailed,
            avifResult_AVIF_RESULT_INVALID_TONE_MAPPED_IMAGE => AvifError::InvalidToneMappedImage,
            other => AvifError::UnknownType(other as u32),
        }
    }
}

impl fmt::Display for AvifError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AvifError::UnknownError => write!(f, "Unknown error"),
            AvifError::InvalidFtyp => write!(f, "Invalid file type"),
            AvifError::NoContent => write!(f, "No content"),
            AvifError::NoYuvFormatSelected => write!(f, "No YUV format selected"),
            AvifError::ReformatFailed => write!(f, "Reformat failed"),
            AvifError::UnsupportedDepth => write!(f, "Unsupported depth"),
            AvifError::EncodeColorFailed => write!(f, "Encode color failed"),
            AvifError::EncodeAlphaFailed => write!(f, "Encode alpha failed"),
            AvifError::BmffParseFailed => write!(f, "BMFF parse failed"),
            AvifError::MissingImageItem => write!(f, "Missing image item"),
            AvifError::DecodeColorFailed => write!(f, "Decode color failed"),
            AvifError::DecodeAlphaFailed => write!(f, "Decode alpha failed"),
            AvifError::ColorAlphaSizeMismatch => write!(f, "Color/alpha size mismatch"),
            AvifError::IspeSizeMismatch => write!(f, "ISPE size mismatch"),
            AvifError::NoCodecAvailable => write!(f, "No codec available"),
            AvifError::NoImagesRemaining => write!(f, "No images remaining"),
            AvifError::InvalidExifPayload => write!(f, "Invalid EXIF payload"),
            AvifError::InvalidImageGrid => write!(f, "Invalid image grid"),
            AvifError::InvalidCodecSpecificOption => write!(f, "Invalid codec specific option"),
            AvifError::TruncatedData => write!(f, "Truncated data"),
            AvifError::IoNotSet => write!(f, "IO not set"),
            AvifError::IoError => write!(f, "IO error"),
            AvifError::WaitingOnIo => write!(f, "Waiting on IO"),
            AvifError::InvalidArgument => write!(f, "Invalid argument"),
            AvifError::NotImplemented => write!(f, "Not implemented"),
            AvifError::OutOfMemory => write!(f, "Out of memory"),
            AvifError::IncompatibleImage => write!(f, "Incompatible image"),
            AvifError::EncodeGainMapFailed => write!(f, "Encode gain map failed"),
            AvifError::DecodeGainMapFailed => write!(f, "Decode gain map failed"),
            AvifError::InvalidToneMappedImage => write!(f, "Invalid tone mapped image"),
            AvifError::UnknownType(code) => write!(f, "Unknown error type: {}", code),
        }
    }
}

impl std::error::Error for AvifError {}
