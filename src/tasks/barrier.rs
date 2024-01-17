use std::sync::Condvar;

pub struct JobBarrier {
    count: usize,
    total: usize,
    condvar: Condvar,
}

impl JobBarrier {
    pub fn new(total: usize) -> Self {
        Self {
            count: 0,
            total,
            condvar: Condvar::new(),
        }
    }

    pub fn notify(&mut self) {
        self.count += 1;

        if self.count == self.total {
            self.condvar.notify_all();
        }
    }

    pub fn wait(&self, lock: std::sync::MutexGuard<'_, Self>) {
        if self.count < self.total {
            std::mem::drop(self.condvar.wait(lock).unwrap());
        }
    }
}
