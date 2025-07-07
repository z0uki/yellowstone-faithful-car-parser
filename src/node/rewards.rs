use crate::node::{DataFrame, Kind, NodeError};

// type Rewards struct {
// 	Kind int
// 	Slot int
// 	Data DataFrame
// }
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Rewards {
    pub slot: u64,
    pub data: DataFrame,
}

impl TryFrom<&[u8]> for Rewards {
    type Error = NodeError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from(serde_cbor::from_slice::<serde_cbor::Value>(value)?)
    }
}

impl TryFrom<serde_cbor::Value> for Rewards {
    type Error = NodeError;

    fn try_from(value: serde_cbor::Value) -> Result<Self, Self::Error> {
        let mut node = Self::default();
        if let serde_cbor::Value::Array(mut vec) = value {
            if let Some(serde_cbor::Value::Integer(kind)) = vec.first() {
                NodeError::assert_invalid_kind(*kind as u64, Kind::Rewards)?;
            }
            if let Some(serde_cbor::Value::Integer(slot)) = vec.get(1) {
                node.slot = *slot as u64;
            }
            if let Some(serde_cbor::Value::Array(data)) = vec.get_mut(2) {
                node.data = DataFrame::try_from(serde_cbor::Value::Array(std::mem::take(data)))?;
            }
        }
        Ok(node)
    }
}

impl Rewards {
    /// Returns whether the rewards data is complete or is split into multiple dataframes.
    pub const fn is_complete(&self) -> bool {
        self.data.next.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        node::{DataFrame, Rewards},
        util::tests::decode_hex,
    };

    #[test]
    fn test_decoding() {
        #[allow(clippy::single_element_loop)]
        for (bytes, frame) in [(
            decode_hex("83051a010114848506f6f6f65528b52ffd04004100000000000000000000bb1bdbca"),
            Rewards {
                slot: 16848004,
                data: DataFrame {
                    hash: None,
                    index: None,
                    total: None,
                    data: decode_hex("28b52ffd04004100000000000000000000bb1bdbca"),
                    next: vec![],
                },
            },
        )] {
            let node = Rewards::try_from(bytes.as_ref()).expect("valid node");
            assert_eq!(node, frame);
        }
    }
}
