//! Transform its representation between structured and byte form.
#![deny(missing_docs, warnings)]
#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(any(feature = "std", test))]
extern crate std;

#[cfg(feature = "alloc")]
use ::alloc::vec::Vec;

macro_rules! test_transformable {
  ($ty: ty => $test_fn:ident($init: expr)) => {
    #[test]
    fn $test_fn() {
      use crate::TestTransformable;

      <$ty>::test_transformable(|| $init);
    }
  };
}

/// The type can transform its representation between structured and byte form.
pub trait Transformable {
  /// The error type returned when encoding or decoding fails.
  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  type Error: std::error::Error;

  /// The error type returned when encoding or decoding fails.
  #[cfg(not(feature = "std"))]
  #[cfg_attr(docsrs, doc(cfg(not(feature = "std"))))]
  type Error: core::fmt::Display;

  /// Encodes the value into the given buffer for transmission.
  ///
  /// Returns the number of bytes written to the buffer.
  fn encode(&self, dst: &mut [u8]) -> Result<usize, Self::Error>;

  /// Encodes the value into a vec for transmission.
  #[cfg(feature = "alloc")]
  fn encode_to_vec(&self) -> Result<Vec<u8>, Self::Error> {
    let mut buf = ::alloc::vec![0u8; self.encoded_len()];
    self.encode(&mut buf)?;
    Ok(buf)
  }

  /// Encodes the value into the given writer for transmission.
  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn encode_to_writer<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize>;

  /// Encodes the value into the given async writer for transmission.
  #[cfg(feature = "async")]
  #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
  fn encode_to_async_writer<W: futures_util::io::AsyncWrite + Send + Unpin>(
    &self,
    writer: &mut W,
  ) -> impl std::future::Future<Output = std::io::Result<usize>> + Send
  where
    Self::Error: Send + Sync + 'static;

  /// Returns the encoded length of the value.
  /// This is used to pre-allocate a buffer for encoding.
  fn encoded_len(&self) -> usize;

  /// Decodes the value from the given buffer received over the wire.
  ///
  /// Returns the number of bytes read from the buffer and the struct.
  fn decode(src: &[u8]) -> Result<(usize, Self), Self::Error>
  where
    Self: Sized;

  /// Decodes the value from the given reader received over the wire.
  ///
  /// Returns the number of bytes read from the reader and the struct.
  #[cfg(feature = "std")]
  #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
  fn decode_from_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<(usize, Self)>
  where
    Self: Sized;

  /// Decodes the value from the given async reader received over the wire.
  ///
  /// Returns the number of bytes read from the reader and the struct.
  #[cfg(feature = "async")]
  #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
  fn decode_from_async_reader<R: futures_util::io::AsyncRead + Send + Unpin>(
    reader: &mut R,
  ) -> impl std::future::Future<Output = std::io::Result<(usize, Self)>> + Send
  where
    Self: Sized,
    Self::Error: Send + Sync + 'static;
}

#[cfg(test)]
trait TestTransformable: Transformable + Eq + core::fmt::Debug + Sized {
  fn test_transformable(init: impl FnOnce() -> Self)
  where
    <Self as Transformable>::Error: core::fmt::Debug,
  {
    let val = init();
    let mut buf = std::vec![0; val.encoded_len()];
    val.encode(&mut buf).unwrap();
    let (_, decoded) = Self::decode(&buf).unwrap();
    assert_eq!(decoded, val);

    #[cfg(feature = "std")]
    {
      let mut buf = std::vec::Vec::new();
      val.encode_to_writer(&mut buf).unwrap();
      let (_, decoded) = Self::decode_from_reader(&mut buf.as_slice()).unwrap();
      assert_eq!(decoded, val);
    }
  }
}

#[cfg(test)]
impl<T: Transformable + Eq + core::fmt::Debug + Sized> TestTransformable for T {}

mod impls;
pub use impls::*;

mod utils;
