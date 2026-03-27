use std::{
    collections::HashSet,
    net::UdpSocket,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::Result;

use crate::intercom::{
    buffer::{BufferEndpoint, BufferReader, BufferWriter},
    config::InternalConfig,
};

const SOCKET_TIMEOUT: Option<Duration> = Some(Duration::from_millis(50));

pub struct NetworkSocket {
    thread_data: Arc<ThreadData>,

    rx_thread: Option<JoinHandle<()>>,
    tx_thread: Option<JoinHandle<()>>,
}

struct ThreadData {
    terminate: AtomicBool,
    socket: UdpSocket,

    packet_size: usize,
    remotes: Mutex<HashSet<String>>,
}

pub fn setup_network_socket(
    buffer: BufferEndpoint,
    config: &InternalConfig,
) -> Result<NetworkSocket> {
    let (rx_buffer, tx_buffer) = buffer.split();

    let thread_data = Arc::new(ThreadData {
        terminate: AtomicBool::new(false),
        socket: UdpSocket::bind(&config.socket_address)?,

        packet_size: config.packet_size,
        remotes: Mutex::new(config.destinations.clone()),
    });

    thread_data.socket.set_read_timeout(SOCKET_TIMEOUT)?;
    thread_data.socket.set_write_timeout(SOCKET_TIMEOUT)?;

    let rx_thread = if config.has_receiver {
        Some(start_rx_thread(thread_data.clone(), rx_buffer)?)
    } else {
        None
    };

    let tx_thread = if config.has_sender {
        Some(start_tx_thread(thread_data.clone(), tx_buffer)?)
    } else {
        None
    };

    Ok(NetworkSocket {
        thread_data,
        rx_thread,
        tx_thread,
    })
}

impl NetworkSocket {
    pub fn can_receive(&self) -> bool {
        self.rx_thread.is_some()
    }

    pub fn can_send(&self) -> bool {
        self.tx_thread.is_some()
    }

    pub fn get_local_address(&self) -> String {
        self.thread_data
            .socket
            .local_addr()
            .map_or(String::default(), |a| a.to_string())
    }

    pub fn get_remote_addresses(&self) -> HashSet<String> {
        let remotes = self
            .thread_data
            .remotes
            .lock()
            .expect("other thread panicked with the lock");

        remotes.clone()
    }

    pub fn add_remote_address(&mut self, address: &str) {
        let mut remotes = self
            .thread_data
            .remotes
            .lock()
            .expect("other thread panicked with the lock");

        remotes.insert(address.to_owned());
    }

    pub fn remove_remote_address(&self, address: &str) {
        let mut remotes = self
            .thread_data
            .remotes
            .lock()
            .expect("other thread panicked with the lock");

        remotes.remove(address);
    }
}

impl Drop for NetworkSocket {
    fn drop(&mut self) {
        self.thread_data.terminate.store(true, Ordering::SeqCst);

        if let Some(handle) = self.rx_thread.take() {
            let _ = handle.join();
        }

        if let Some(handle) = self.tx_thread.take() {
            let _ = handle.join();
        }
    }
}

fn start_rx_thread(data: Arc<ThreadData>, mut rx_buffer: BufferWriter) -> Result<JoinHandle<()>> {
    let builder = thread::Builder::new().name(String::from("rx_thread"));

    let handle = builder.spawn(move || {
        let ThreadData {
            terminate,
            socket,
            packet_size,
            ..
        } = &*data;

        let mut packet = vec![0u8; packet_size * size_of::<f32>()].into_boxed_slice();

        while !terminate.load(Ordering::SeqCst) {
            let Ok((length, _)) = socket.recv_from(&mut packet) else {
                continue;
            };

            let buffer: Box<[f32]> = packet[..length]
                .as_chunks::<4>()
                .0
                .iter()
                .map(|f| f32::from_be_bytes(*f))
                .collect();

            rx_buffer.push_partial_slice(&buffer);
        }
    })?;

    Ok(handle)
}

fn start_tx_thread(data: Arc<ThreadData>, mut tx_buffer: BufferReader) -> Result<JoinHandle<()>> {
    let builder = thread::Builder::new().name(String::from("tx_thread"));

    let handle = builder.spawn(move || {
        let ThreadData {
            terminate,
            socket,
            packet_size,
            remotes,
        } = &*data;

        while !terminate.load(Ordering::SeqCst) {
            let Ok(chunk) = tx_buffer.read_chunk(*packet_size) else {
                thread::sleep(Duration::from_millis(1)); // TODO
                continue;
            };

            let packet: Box<[u8]> = chunk
                .into_iter()
                .flat_map(|f| f32::to_be_bytes(f))
                .collect();

            let remotes = remotes.lock().expect("other thread panicked with the lock");
            for addr in remotes.iter() {
                if let Err(err) = socket.send_to(&packet, addr) {
                    eprintln!("encountered error while while sending to {addr}, err={err}");
                };
            }
        }
    })?;

    Ok(handle)
}
