use bytes::Bytes;

/// Runtime-variable sharing, real lexical peer Drops, surviving final owner.
pub fn finite_shared_scope(input: Box<[u8]>, count: usize) -> Vec<u8> {
    let survivor = {
        let original = Bytes::from(input);
        original.clone()
    };
    let mut owners = Vec::new();
    let mut made = 0;
    while made < count {
        owners.push(survivor.clone());
        made += 1;
    }
    while let Some(peer) = owners.pop() {
        // peer's lexical normal Drop retires its actual ticket.
    }
    let observed = AsRef::<[u8]>::as_ref(&survivor).to_vec();
    observed
}
