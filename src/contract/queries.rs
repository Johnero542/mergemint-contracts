use crate::contract::state::{BountyId, BountyMeta, BOUNTIES};
use cosmwasm_std::{Deps, Env, StdResult};

pub fn get_bounty_meta(deps: Deps, _env: Env, id: BountyId) -> StdResult<Option<BountyMeta>> {
    BOUNTIES.may_load(deps.storage, id)
}

pub fn get_bounty_metas(deps: Deps, _env: Env, ids: Vec<BountyId>) -> StdResult<Vec<Option<BountyMeta>>> {
    ids.into_iter()
        .map(|id| BOUNTIES.may_load(deps.storage, id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use crate::contract::state::BOUNTIES;

    #[test]
    fn test_get_bounty_metas_batch() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let meta1 = BountyMeta {
            title: "Bounty 1".to_string(),
            description: "Description 1".to_string(),
        };
        let meta3 = BountyMeta {
            title: "Bounty 3".to_string(),
            description: "Description 3".to_string(),
        };

        BOUNTIES.save(deps.as_mut().storage, 1u64, &meta1).unwrap();
        BOUNTIES.save(deps.as_mut().storage, 3u64, &meta3).unwrap();

        let ids = vec![1u64, 2u64, 3u64, 4u64];
        let result = get_bounty_metas(deps.as_ref(), env, ids).unwrap();

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], Some(meta1));
        assert_eq!(result[1], None);
        assert_eq!(result[2], Some(meta3));
        assert_eq!(result[3], None);
    }

    #[test]
    fn test_get_bounty_metas_empty() {
        let deps = mock_dependencies();
        let env = mock_env();

        let ids = vec![1u64, 2u64, 3u64];
        let result = get_bounty_metas(deps.as_ref(), env, ids).unwrap();

        assert_eq!(result.len(), 3);
        assert!(result.iter().all(|r| r.is_none()));
    }

    #[test]
    fn test_get_bounty_metas_all_exist() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let meta1 = BountyMeta {
            title: "Bounty 1".to_string(),
            description: "Description 1".to_string(),
        };
        let meta2 = BountyMeta {
            title: "Bounty 2".to_string(),
            description: "Description 2".to_string(),
        };

        BOUNTIES.save(deps.as_mut().storage, 1u64, &meta1).unwrap();
        BOUNTIES.save(deps.as_mut().storage, 2u64, &meta2).unwrap();

        let ids = vec![1u64, 2u64];
        let result = get_bounty_metas(deps.as_ref(), env, ids).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Some(meta1));
        assert_eq!(result[1], Some(meta2));
    }
}
