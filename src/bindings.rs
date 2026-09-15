#[link(wasm_import_module = "conn")]
unsafe extern "C" {
    /// Host function for setting the map of the peers that should stay connected.
    ///
    /// Must be called before exit.
    ///
    /// If zero is passed, the multipalyer is cancelled.
    pub(crate) unsafe fn set_peers(peer_map: u32);

    /// Host function for marking the connection as ready.
    ///
    /// Send the ready request to the given peer map.
    pub(crate) unsafe fn set_ready(peer_map: u32, hash: u32) -> u32;

    // Host function for getting the map of peers that sent the ready message.
    pub(crate) unsafe fn get_ready_map(peer_map: u32, hash: u32) -> u32;
}
