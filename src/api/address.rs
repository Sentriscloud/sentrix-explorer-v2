//! Address classifier — picks the right provider based on shape alone.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AddressKind {
    /// `0x` + 40 hex chars. EVM H160. Routed to `EvmProvider`.
    Evm(String),
    /// 32-byte hex hash (block, tx, state root). Routed to
    /// `NativeProvider::get_block` or similar.
    Hash32(String),
    /// Plain block height (decimal u64). Routed to
    /// `NativeProvider::get_block_by_height`.
    BlockHeight(u64),
    /// Anything else — surface as a search miss.
    Unknown,
}

pub fn classify_address(input: &str) -> AddressKind {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return AddressKind::Unknown;
    }

    if let Ok(h) = trimmed.parse::<u64>() {
        return AddressKind::BlockHeight(h);
    }

    let cleaned = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);

    let is_hex = !cleaned.is_empty() && cleaned.bytes().all(|b| b.is_ascii_hexdigit());
    if !is_hex {
        return AddressKind::Unknown;
    }

    match cleaned.len() {
        40 => AddressKind::Evm(format!("0x{}", cleaned.to_lowercase())),
        64 => AddressKind::Hash32(cleaned.to_lowercase()),
        _ => AddressKind::Unknown,
    }
}
