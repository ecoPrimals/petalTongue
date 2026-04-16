// SPDX-License-Identifier: AGPL-3.0-or-later
//! Length-prefixed BTSP wire framing.

/// Maximum BTSP frame size (16 MiB).
pub(super) const MAX_FRAME_SIZE: u32 = 0x0100_0000;

/// Read a length-prefixed BTSP frame.
pub async fn read_frame<R: tokio::io::AsyncReadExt + Unpin>(
    reader: &mut R,
) -> Result<bytes::Bytes, std::io::Error> {
    let len = reader.read_u32().await?;
    if len > MAX_FRAME_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("BTSP frame too large: {len} > {MAX_FRAME_SIZE}"),
        ));
    }
    let mut buf = bytes::BytesMut::zeroed(len as usize);
    reader.read_exact(&mut buf).await?;
    Ok(buf.freeze())
}

/// Write a length-prefixed BTSP frame.
pub async fn write_frame<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    data: &[u8],
) -> Result<(), std::io::Error> {
    let len = u32::try_from(data.len()).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "frame too large for u32")
    })?;
    writer.write_u32(len).await?;
    writer.write_all(data).await?;
    writer.flush().await
}
