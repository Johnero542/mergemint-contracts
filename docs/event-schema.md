# Event Schema

Contract events emitted for indexer integration.

## Events

### bounty_created
- **Topics**: `(Symbol("bounty_created"), creator_address)`
- **Data**: `(bounty_id, reward_amount)`
- **Trigger**: New bounty created
- **Purpose**: Notify indexer of new bounty

### bounty_claimed
- **Topics**: `(Symbol("bounty_claimed"), contributor_address)`
- **Data**: `bounty_id`
- **Trigger**: Contributor claims bounty
- **Purpose**: Notify indexer of bounty assignment

### bounty_updated
- **Topics**: `(Symbol("bounty_updated"), creator_address)`
- **Data**: `bounty_id`
- **Trigger**: Bounty metadata updated
- **Purpose**: Trigger indexer cache refresh

### bounty_disputed
- **Topics**: `(Symbol("bounty_disputed"), caller_address)`
- **Data**: `bounty_id`
- **Trigger**: Dispute raised on bounty
- **Purpose**: Notify indexer of dispute

### bounty_completed
- **Topics**: `(Symbol("bounty_completed"), contributor_address)`
- **Data**: `bounty_id`
- **Trigger**: Bounty completed and reward paid
- **Purpose**: Notify indexer of completion

### reward_paid
- **Topics**: `(Symbol("reward_paid"), contributor_address)`
- **Data**: `(bounty_id, amount)`
- **Trigger**: Token transfer confirmed
- **Purpose**: Confirm payment to indexer
