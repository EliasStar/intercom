use std::collections::HashSet;

use anyhow::{Result, bail};

use crate::intercom::{
    audio::{AudioClient, setup_audio_client},
    buffer::create_duplex_buffer,
    config::{IntercomConfig, create_internal_config},
    network::{NetworkSocket, setup_network_socket},
};

pub struct IntercomHandle {
    #[expect(unused)]
    audio_client: AudioClient,
    network_socket: NetworkSocket,
}

pub fn run_intercom(config: IntercomConfig) -> Result<IntercomHandle> {
    let config = create_internal_config(config)?;

    let (audio_buffer, network_buffer) = create_duplex_buffer(config.buffer_size);

    Ok(IntercomHandle {
        audio_client: setup_audio_client(audio_buffer)?,
        network_socket: setup_network_socket(network_buffer, &config)?,
    })
}

impl IntercomHandle {
    pub fn get_receiver_address(&self) -> Result<String> {
        if !self.network_socket.can_receive() {
            bail!("intercom is send only")
        }

        Ok(self.network_socket.get_local_address())
    }

    pub fn get_destination_addresses(&self) -> Result<HashSet<String>> {
        if !self.network_socket.can_send() {
            bail!("intercom is receive only")
        }

        Ok(self.network_socket.get_remote_addresses())
    }

    pub fn add_destination_address(&mut self, address: &str) -> Result<()> {
        if !self.network_socket.can_send() {
            bail!("intercom is receive only")
        }

        self.network_socket.add_remote_address(address);
        Ok(())
    }

    pub fn remove_destination_address(&self, address: &str) -> Result<()> {
        if !self.network_socket.can_send() {
            bail!("intercom is receive only")
        }

        self.network_socket.remove_remote_address(address);
        Ok(())
    }
}
