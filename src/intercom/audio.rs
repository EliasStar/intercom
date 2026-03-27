use anyhow::Result;
use jack::{AsyncClient, AudioIn, AudioOut, Client, ClientOptions, Control, Port, ProcessHandler};

use crate::intercom::buffer::{BufferEndpoint, BufferReader, BufferWriter};

pub struct AudioClient(#[expect(unused)] AsyncClient<(), IntercomProcessHandler>);

pub fn setup_audio_client(buffer: BufferEndpoint) -> Result<AudioClient> {
    let (client, status) = Client::new("intercom", ClientOptions::default())?;
    if !status.is_empty() {
        eprintln!(
            "encountered abnormal status while creating jack client, status={}",
            status.bits()
        );
    }

    let (in_buffer, out_buffer) = buffer.split();

    let process_handler = IntercomProcessHandler {
        in_port: client.register_port("in", AudioIn::default())?,
        out_port: client.register_port("out", AudioOut::default())?,
        in_buffer,
        out_buffer,
    };

    let in_name = process_handler.in_port.name()?;
    let out_name = process_handler.out_port.name()?;

    let client = client.activate_async((), process_handler)?;

    client
        .as_client()
        .connect_ports_by_name("system:capture_1", &in_name)?;
    client
        .as_client()
        .connect_ports_by_name(&out_name, "system:playback_1")?;
    client
        .as_client()
        .connect_ports_by_name(&out_name, "system:playback_2")?;

    Ok(AudioClient(client))
}

struct IntercomProcessHandler {
    in_port: Port<AudioIn>,
    out_port: Port<AudioOut>,

    in_buffer: BufferWriter,
    out_buffer: BufferReader,
}

impl ProcessHandler for IntercomProcessHandler {
    fn process(&mut self, _client: &Client, scope: &jack::ProcessScope) -> Control {
        let in_data = self.in_port.as_slice(scope);
        self.in_buffer.push_partial_slice(in_data);

        let out_data = self.out_port.as_mut_slice(scope);
        out_data.fill(0.0);
        self.out_buffer.pop_partial_slice(out_data);

        Control::Continue
    }
}
