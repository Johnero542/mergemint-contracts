use stellar_xdr::curr::{ScVal, ScVec};
use anyhow::{anyhow, Result};

pub fn decode_u64(val: &ScVal) -> Result<u64> {
    match val {
        ScVal::U64(u) => Ok(u.0),
        _ => Err(anyhow!("Expected U64, got {:?}", val)),
    }
}

pub fn decode_bounty_id_list(val: &ScVal) -> Result<Vec<u64>> {
    match val {
        ScVal::Vec(Some(vec)) => {
            vec.iter()
                .map(|item| decode_u64(item))
                .collect::<Result<Vec<u64>>>()
        }
        ScVal::Vec(None) => Ok(vec![]),
        _ => Err(anyhow!("Expected Vec, got {:?}", val)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stellar_xdr::curr::U64;

    #[test]
    fn test_decode_u64() {
        let val = ScVal::U64(U64(42));
        assert_eq!(decode_u64(&val).unwrap(), 42);
    }

    #[test]
    fn test_decode_u64_invalid() {
        let val = ScVal::Bool(true);
        assert!(decode_u64(&val).is_err());
    }

    #[test]
    fn test_decode_bounty_id_list() {
        let ids = vec![ScVal::U64(U64(1)), ScVal::U64(U64(2)), ScVal::U64(U64(3))];
        let val = ScVal::Vec(Some(ScVec::try_from(ids).unwrap()));
        let result = decode_bounty_id_list(&val).unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_decode_bounty_id_list_empty() {
        let val = ScVal::Vec(None);
        let result = decode_bounty_id_list(&val).unwrap();
        assert_eq!(result, Vec::<u64>::new());
    }

    #[test]
    fn test_decode_bounty_id_list_invalid() {
        let val = ScVal::Bool(true);
        assert!(decode_bounty_id_list(&val).is_err());
    }
}
