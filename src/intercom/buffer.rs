use rtrb::{Consumer, Producer, RingBuffer};

pub type BufferWriter = Producer<f32>;
pub type BufferReader = Consumer<f32>;

pub struct BufferEndpoint {
    writer: BufferWriter,
    reader: BufferReader,
}

impl BufferEndpoint {
    pub fn split(self) -> (BufferWriter, BufferReader) {
        (self.writer, self.reader)
    }
}

pub fn create_duplex_buffer(size: usize) -> (BufferEndpoint, BufferEndpoint) {
    let (e1_writer, e2_reader) = RingBuffer::<f32>::new(size);
    let (e2_writer, e1_reader) = RingBuffer::<f32>::new(size);

    (
        BufferEndpoint {
            writer: e1_writer,
            reader: e1_reader,
        },
        BufferEndpoint {
            writer: e2_writer,
            reader: e2_reader,
        },
    )
}
