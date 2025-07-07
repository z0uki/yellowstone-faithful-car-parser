use {
    crate::node::{Kind, NodeError},
    cid::Cid,
};

// type DataFrame struct {
// 	Kind  int
// 	Hash  **int
// 	Index **int
// 	Total **int
// 	Data  []uint8
// 	Next  **List__Link
// }
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct DataFrame {
    pub hash: Option<u64>,
    pub index: Option<u64>,
    pub total: Option<u64>,
    pub data: Vec<u8>,
    pub next: Vec<Cid>,
}

impl TryFrom<&[u8]> for DataFrame {
    type Error = NodeError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from(serde_cbor::from_slice::<serde_cbor::Value>(value)?)
    }
}

impl TryFrom<serde_cbor::Value> for DataFrame {
    type Error = NodeError;

    fn try_from(value: serde_cbor::Value) -> Result<Self, Self::Error> {
        let mut node = Self::default();
        if let serde_cbor::Value::Array(mut vec) = value {
            if let Some(serde_cbor::Value::Integer(kind)) = vec.first() {
                NodeError::assert_invalid_kind(*kind as u64, Kind::DataFrame)?;
            }
            if let Some(serde_cbor::Value::Integer(hash)) = vec.get(1) {
                node.hash = Some(*hash as u64);
            }
            if let Some(serde_cbor::Value::Integer(index)) = vec.get(2) {
                node.index = Some(*index as u64);
            }
            if let Some(serde_cbor::Value::Integer(total)) = vec.get(3) {
                node.total = Some(*total as u64);
            }
            if let Some(serde_cbor::Value::Bytes(data)) = vec.get_mut(4) {
                node.data = std::mem::take(data);
            }

            if let Some(serde_cbor::Value::Array(next)) = &vec.get(5) {
                for cid in next {
                    if let serde_cbor::Value::Bytes(cid) = cid {
                        node.next.push(Cid::try_from(&cid[1..])?);
                    }
                }
            }
        }
        Ok(node)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        node::DataFrame,
        util::tests::{decode_cids, decode_hex},
    };

    #[test]
    fn test_decoding() {
        for (bytes, frame) in [
            (
                decode_hex("86063b4630c0a8d52653c101024620776f726c6480"),
                DataFrame {
                    hash: Some(13388989860809387070),
                    index: Some(1),
                    total: Some(2),
                    data: decode_hex("20776f726c64"),
                    next: vec![],
                },
            ),
            (
                decode_hex("86061b48acf56598bd34f8181a181c4ab24fe965f006c911090e80"),
                DataFrame {
                    hash: Some(5236830283428082936),
                    index: Some(26),
                    total: Some(28),
                    data: decode_hex("b24fe965f006c911090e"),
                    next: vec![],
                },
            ),
            (
                decode_hex(
                    "86061b48acf56598bd34f816181c4a6fedb3ada52763ab71e985d82a582500017112207a470286e1843dbaa2ffb81d30018a40e8c3bb14026b6085fd63d49fd6eb1fb0d82a582500017112201c8cb9aa3b528a23d7d53a8ee3521f9223e6a791f3d6bb88e01fcae192f5e5c6d82a582500017112206bc71f7272fb4138de6cf336b63fc2b23dc5450480473e74de2b69fa0eb6af3cd82a582500017112205732ff009530b6506437a05cc070885fba4da69ff40bd30c6febbb7c1d349266d82a5825000171122051d872d71e7a36e28bc4361c852c805bc7102f2989bed661966c41f2d933314f",
                ),
                DataFrame {
                    hash: Some(5236830283428082936),
                    index: Some(22),
                    total: Some(28),
                    data: decode_hex("6fedb3ada52763ab71e9"),
                    next: decode_cids([
                        "bafyreid2i4binymehw5kf75yduyadcsa5db3wfacnnqil7ld2sp5n2y7wa",
                        "bafyreia4rs42uo2srir5pvj2r3rveh4septkpept225yrya7zlqzf5pfyy",
                        "bafyreidly4pxe4x3ie4n43htg23d7qvshxcukbeai47hjxrlnh5a5nvphq",
                        "bafyreicxgl7qbfjqwzigin5altahbcc7xjg2nh7ubpjqy37lxn6b2nesmy",
                        "bafyreicr3bznoht2g3rixrbwdscszac3y4ic6kmjx3lgdftmihznsmzrj4",
                    ]),
                },
            ),
        ] {
            let node = DataFrame::try_from(bytes.as_ref()).expect("valid node");
            assert_eq!(node, frame);
        }
    }
}
