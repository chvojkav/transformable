use super::*;

#[cfg(any(feature = "alloc", feature = "std"))]
mod bytes;

#[cfg(any(feature = "alloc", feature = "std"))]
mod string;

#[cfg(any(feature = "alloc", feature = "std"))]
mod vec;

#[cfg(feature = "std")]
mod net;
#[cfg(feature = "std")]
pub use net::*;

mod time;
pub use time::*;

mod numbers;
pub use numbers::*;

#[cfg(feature = "smallvec")]
mod smallvec;

mod bytes_array;

/// The error type for errors that get returned when encoding or decoding fails.
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum BytesTransformError {
  /// Returned when the buffer is too small to encode.
  #[cfg_attr(feature = "std", error(
    "buffer is too small, use `Transformable::encoded_len` to pre-allocate a buffer with enough space"
  ))]
  EncodeBufferTooSmall,
  /// Returned when the bytes are corrupted.
  #[cfg_attr(feature = "std", error("corrupted"))]
  Corrupted,
}

#[cfg(not(feature = "std"))]
impl core::fmt::Display for BytesTransformError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::EncodeBufferTooSmall => write!(
        f,
        "buffer is too small, use `Transformable::encoded_len` to pre-allocate a buffer with enough space"
      ),
      Self::Corrupted => write!(f, "corrupted"),
    }
  }
}

#[cfg(any(feature = "alloc", feature = "std"))]
const LEGNTH_SIZE: usize = core::mem::size_of::<u32>();

// inlined max 64 bytes on stack when transforming
#[cfg(feature = "std")]
const INLINED: usize = 256;

#[cfg(all(feature = "std", feature = "async"))]
async fn decode_bytes_from_async<R: futures_util::io::AsyncRead + Unpin>(
  src: &mut R,
) -> std::io::Result<(usize, Vec<u8>)> {
  use futures_util::io::AsyncReadExt;

  let mut len_buf = [0u8; LEGNTH_SIZE];
  src.read_exact(&mut len_buf).await?;
  let len = u32::from_be_bytes(len_buf) as usize;
  let mut buf = vec![0u8; len];
  src
    .read_exact(&mut buf)
    .await
    .map(|_| (len + LEGNTH_SIZE, buf))
}

#[cfg(feature = "std")]
fn decode_bytes_from<R: std::io::Read>(src: &mut R) -> std::io::Result<(usize, Vec<u8>)> {
  let mut len_buf = [0u8; LEGNTH_SIZE];
  src.read_exact(&mut len_buf)?;
  let len = u32::from_be_bytes(len_buf) as usize;
  let mut buf = vec![0u8; len];
  src.read_exact(&mut buf).map(|_| (LEGNTH_SIZE + len, buf))
}

#[cfg(any(feature = "alloc", feature = "std"))]
fn decode_bytes(src: &[u8]) -> Result<(usize, Vec<u8>), ()> {
  let len = src.len();
  if len < LEGNTH_SIZE {
    return Err(());
  }

  let data_len = u32::from_be_bytes([src[0], src[1], src[2], src[3]]) as usize;
  if data_len > len - LEGNTH_SIZE {
    return Err(());
  }

  let total_len = LEGNTH_SIZE + data_len;
  Ok((total_len, src[LEGNTH_SIZE..LEGNTH_SIZE + data_len].to_vec()))
}

#[cfg(any(feature = "alloc", feature = "std"))]
fn encode_bytes(src: &[u8], dst: &mut [u8]) -> Result<(), ()> {
  let encoded_len = encoded_bytes_len(src);
  if dst.len() < encoded_len {
    return Err(());
  }
  let src_len = src.len();
  dst[..LEGNTH_SIZE].copy_from_slice(&(src_len as u32).to_be_bytes());
  dst[LEGNTH_SIZE..LEGNTH_SIZE + src_len].copy_from_slice(src);
  Ok(())
}

#[cfg(feature = "std")]
fn encode_bytes_to<W: std::io::Write>(src: &[u8], dst: &mut W) -> std::io::Result<()> {
  let len = src.len();
  let len_bytes = (len as u32).to_be_bytes();
  if len + LEGNTH_SIZE <= INLINED {
    let mut buf = [0u8; INLINED];
    buf[..LEGNTH_SIZE].copy_from_slice(&len_bytes);
    buf[LEGNTH_SIZE..LEGNTH_SIZE + len].copy_from_slice(src);
    dst.write_all(&buf[..LEGNTH_SIZE + len])
  } else {
    let mut buf = std::vec![0; LEGNTH_SIZE + len];
    buf[..LEGNTH_SIZE].copy_from_slice(&len_bytes);
    buf[LEGNTH_SIZE..].copy_from_slice(src);
    dst.write_all(&buf)
  }
}

#[cfg(all(feature = "std", feature = "async"))]
async fn encode_bytes_to_async<W: futures_util::io::AsyncWrite + Unpin>(
  src: &[u8],
  dst: &mut W,
) -> std::io::Result<()> {
  use futures_util::io::AsyncWriteExt;

  let len = src.len();
  let len_bytes = (len as u32).to_be_bytes();
  if len + LEGNTH_SIZE <= INLINED {
    let mut buf = [0u8; INLINED];
    buf[..LEGNTH_SIZE].copy_from_slice(&len_bytes);
    buf[LEGNTH_SIZE..LEGNTH_SIZE + len].copy_from_slice(src);
    dst.write_all(&buf[..LEGNTH_SIZE + len]).await
  } else {
    let mut buf = std::vec![0; LEGNTH_SIZE + len];
    buf[..LEGNTH_SIZE].copy_from_slice(&len_bytes);
    buf[LEGNTH_SIZE..].copy_from_slice(src);
    dst.write_all(&buf).await
  }
}

#[cfg(any(feature = "alloc", feature = "std"))]
fn encoded_bytes_len(src: &[u8]) -> usize {
  LEGNTH_SIZE + src.len()
}
