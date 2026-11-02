fn net_from_client(player: u128, channel: &str, data: &[u8]) {
    let _ = (player, data);
    if channel != "redpower_2:control" && channel != "horizon:mod_net" {
        return;
    }
    // Receives 6502 Monitor terminal keystrokes and disk drive packets
}
