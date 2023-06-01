use std::sync::{Condvar, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct  BarrierError;

pub struct  CyclicBarrier<T>{
    t_counter : std::sync::Mutex<(usize, bool)>,
    cv : std::sync::Condvar,
    value : T,
    t_counter_expected : usize
}

impl<T : Clone> CyclicBarrier<T>{

    pub fn new(t_counter : usize, value : T ) -> Self
    {
        Self{
            t_counter: std::sync::Mutex::new( (0, false)),
            cv: Condvar::new(),
            value,
            t_counter_expected: t_counter
        }
    }

    pub fn wait(& self) -> ()
    {
        let mut t_counter = self.t_counter.lock()
            .expect("Impossibile acquisire il lock");

        while t_counter.1 {
            t_counter = self.cv.wait(t_counter ).unwrap();
        }

        t_counter.0 += 1;

        if t_counter.0 == self.t_counter_expected
        {
            t_counter.1 = true;
            self.cv.notify_all();
        }

        while !t_counter.1 {
            t_counter = self.cv.wait(t_counter ).unwrap();
        }

        t_counter.0 -= 1;

        if t_counter.0 == 0
        {
            t_counter.1 = false;
            self.cv.notify_all();
        }

    }

}