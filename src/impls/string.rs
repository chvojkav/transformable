use super::*;

#[cfg(not(feature = "std"))]
use ::alloc::{boxed::Box, string::String};

use ::alloc::sync::Arc;

use core::borrow::Borrow;

/// The error type for errors that get returned when encoding or decoding str based structs fails.
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum StringTransformError {
  /// Returned when the buffer is too small to encode.
  #[cfg_attr(feature = "std", error(
    "buffer is too small, use `Transformable::encoded_len` to pre-allocate a buffer with enough space"
  ))]
  EncodeBufferTooSmall,
  /// Returned when the decoding meet corruption.
  #[cfg_attr(feature = "std", error("not enough bytes to decode"))]
  NotEnoughBytes,
  /// Returned when the decoding meet utf8 error.
  #[cfg_attr(feature = "std", error("{0}"))]
  Utf8Error(#[cfg_attr(feature = "std", from)] core::str::Utf8Error),
}

#[cfg(all(not(feature = "std"), feature = "alloc"))]
impl core::convert::From<core::str::Utf8Error> for StringTransformError {
  fn from(err: core::str::Utf8Error) -> Self {
    Self::Utf8Error(err)
  }
}

#[cfg(all(not(feature = "std"), feature = "alloc"))]
impl core::fmt::Display for StringTransformError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::EncodeBufferTooSmall => write!(
        f,
        "buffer is too small, use `Transformable::encoded_len` to pre-allocate a buffer with enough space"
      ),
      Self::NotEnoughBytes => write!(f, "not enough bytes to decode"),
      Self::Utf8Error(val) => write!(f, "{val}"),
    }
  }
}

macro_rules! impl_string {
  ($ty: ty => $test_fn:ident($init: expr)) => {
    impl Transformable for $ty {
      type Error = StringTransformError;

      fn encode(&self, dst: &mut [u8]) -> Result<usize, Self::Error> {
        let src: &str = self.borrow();
        encode_bytes(src.as_bytes(), dst).map_err(|_| Self::Error::EncodeBufferTooSmall)
      }

      #[cfg(feature = "std")]
      #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
      fn encode_to_writer<W: std::io::Write>(&self, dst: &mut W) -> std::io::Result<usize> {
        let src: &str = self.borrow();
        encode_bytes_to(src.as_bytes(), dst)
      }

      #[cfg(feature = "async")]
      #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
      async fn encode_to_async_writer<W: futures_util::io::AsyncWrite + Send + Unpin>(
        &self,
        dst: &mut W,
      ) -> std::io::Result<usize> {
        let src: &str = self.borrow();
        encode_bytes_to_async(src.as_bytes(), dst).await
      }

      fn encoded_len(&self) -> usize {
        let src: &str = self.borrow();
        encoded_bytes_len(src.as_bytes())
      }

      fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
      where
        Self: Sized,
      {
        decode_bytes(src)
          .map_err(|_| Self::Error::NotEnoughBytes)
          .and_then(|(readed, bytes)| {
            core::str::from_utf8(bytes.as_ref())
              .map(|s| (readed, Self::from(s)))
              .map_err(Into::into)
          })
      }

      #[cfg(feature = "std")]
      #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
      fn decode_from_reader<R: std::io::Read>(src: &mut R) -> std::io::Result<(usize, Self)>
      where
        Self: Sized,
      {
        decode_bytes_from(src).and_then(|(readed, bytes)| {
          core::str::from_utf8(bytes.as_ref())
            .map(|s| (readed, Self::from(s)))
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
        })
      }

      #[cfg(feature = "async")]
      #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
      async fn decode_from_async_reader<R: futures_util::io::AsyncRead + Send + Unpin>(
        src: &mut R,
      ) -> std::io::Result<(usize, Self)>
      where
        Self: Sized,
      {
        decode_bytes_from_async(src)
          .await
          .and_then(|(readed, bytes)| {
            core::str::from_utf8(bytes.as_ref())
              .map(|s| (readed, Self::from(s)))
              .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
          })
      }
    }

    test_transformable!($ty => $test_fn($init));
  };
}

impl_string!(String => test_string_transformable(String::from("hello world")));

#[cfg(feature = "smol_str")]
impl_string!(smol_str::SmolStr => test_smol_str_transformable(smol_str::SmolStr::from("hello world")));

impl_string!(Box<str> => test_box_str_transformable(Box::from("hello world")));

impl_string!(Arc<str> => test_arc_str_transformable(Arc::from("hello world")));
