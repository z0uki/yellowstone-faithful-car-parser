use {
    std::io,
    tokio::io::{AsyncRead, AsyncReadExt},
};

#[inline]
pub async fn read_exact<R: AsyncRead + Unpin>(reader: &mut R, size: usize) -> io::Result<Vec<u8>> {
    let mut buffer = Vec::with_capacity(size);
    // SAFETY: All `size` bytes would be initialized by read_exact
    #[allow(clippy::uninit_vec)]
    unsafe {
        buffer.set_len(size)
    }
    reader.read_exact(&mut buffer).await?;
    Ok(buffer)
}

#[cfg(test)]
pub mod tests {
    use {cid::Cid, const_hex::decode};

    pub fn decode_hex<T: AsRef<[u8]>>(input: T) -> Vec<u8> {
        decode(input).unwrap()
    }

    pub fn decode_cid(cid: &'static str) -> Cid {
        Cid::try_from(cid).unwrap()
    }

    pub fn decode_cids<const N: usize>(cids: [&'static str; N]) -> Vec<Cid> {
        cids.into_iter().map(decode_cid).collect()
    }
}
