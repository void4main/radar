
struct RingBuffer<T> {
    buffer: Vec<Option<T>>,
    read_head: usize,
    write_head: usize,
    buffer_size: usize,
}

pub impl<T: Clone + std::fmt::Debug> RingBuffer<T> {
    fn new(buffer_size: usize) -> Self {
        RingBuffer {
            buffer: vec![None; buffer_size],
            read_head: buffer_size - 1,
            write_head: buffer_size - 1,
            buffer_size: buffer_size,
        }
    }

    fn push(&mut self, element: T) {
        self.write_head = self.next_head_pos(self.write_head);
        self.buffer[self.write_head] = Some(element);
    }

    fn pull(&mut self) -> Option<T> {
        self.read_head = self.next_head_pos(self.read_head);
        let value = &self.buffer[self.read_head];
        value.clone()
    }

    fn next_head_pos(&self, pos_now: usize) -> usize {
        (pos_now + 1) % self.buffer_size
    } 

    fn print(&self) {
        println!("{:?}", self.buffer);
    }
}
