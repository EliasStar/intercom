use std::collections::HashSet;

use anyhow::{Result, bail};

pub struct IntercomConfig {
    /// None => no receiver;
    /// Some(0) => receive on random port;
    /// Some(x) => receive on port x;
    receiver_port: Option<u16>,

    /// list of all remote receivers in the form of "host:port" destination addresses. if empty intercom will be receive only.
    destinations: HashSet<String>,

    /// false => bind to all interfaces;
    /// true => bind to localhost only;
    private: bool,

    /// buffer size in samples
    buffer_size: usize,

    /// packet size in samples
    packet_size: usize,
}

impl Default for IntercomConfig {
    fn default() -> Self {
        Self {
            receiver_port: None,
            destinations: HashSet::new(),
            private: false,
            buffer_size: 4096,
            packet_size: 24,
        }
    }
}

impl IntercomConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_port(receiver_port: u16) -> Self {
        Self::default().receive_on(receiver_port)
    }

    pub fn receive_on(mut self, receiver_port: u16) -> Self {
        self.receiver_port = Some(receiver_port);
        self
    }

    pub fn send_to(mut self, destination_address: String) -> Self {
        self.destinations.insert(destination_address);
        self
    }

    pub fn on_localhost_only(mut self) -> Self {
        self.private = true;
        self
    }

    pub fn set_buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }
}

pub fn create_internal_config(config: IntercomConfig) -> Result<InternalConfig> {
    let has_receiver = config.receiver_port.is_some();
    let has_sender = !config.destinations.is_empty();

    if !has_receiver && !has_sender {
        bail!("intercom neither sends nor receives");
    }

    let port = config.receiver_port.unwrap_or(0);
    let socket_address = if config.private {
        format!("127.0.0.1:{port}")
    } else {
        format!("0.0.0.0:{port}")
    };

    Ok(InternalConfig {
        has_receiver,
        has_sender,

        socket_address,
        destinations: config.destinations,

        buffer_size: config.buffer_size,
        packet_size: config.packet_size,
    })
}

pub struct InternalConfig {
    pub has_receiver: bool,
    pub has_sender: bool,

    pub socket_address: String,
    pub destinations: HashSet<String>,

    pub buffer_size: usize,
    pub packet_size: usize,
}
