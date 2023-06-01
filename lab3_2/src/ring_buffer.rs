use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

const BUFFER_MAX_SIZE : usize = 10;

pub struct RingBuffer<T>{
    /*
        Circular buffer implemented as a mutex variables
    */
    buffer : std::sync::Mutex<VecDeque<T>>,
    cv : std::sync::Condvar
}

pub struct CircularBufferError;

impl<T> RingBuffer<T>{

    pub fn new() -> Self
    {
        Self{
            buffer: Mutex::new(VecDeque::<T>::new() ),
            cv: Condvar::new()
        }
    }

    pub fn read(& self ) -> Option<T>{
        let mut buffer = self.buffer.lock().unwrap();

        /* Versione1 - utilizzo di API del OS
        while buffer.is_empty() {
            buffer =  self.cv.wait(buffer).unwrap();
        } */

        buffer = self.cv.wait_while(buffer , |buffer| {buffer.is_empty()}).unwrap();

        /*
            Extract for queue
        */
        let result = (*buffer).pop_back();

        if buffer.is_empty(){
            self.cv.notify_one();
        }

        result
    }

    pub fn write(&self , value : T ) -> Result< () , CircularBufferError >
    {
        let mut buffer = self.buffer.lock().unwrap();

        if ! buffer.is_empty() {
            buffer = self.cv.wait_while(buffer , |buffer| {buffer.is_empty()}).unwrap();
        }

        (*buffer).push_front(value);

        if buffer.len() == BUFFER_MAX_SIZE{
            self.cv.notify_one();
            return Err(CircularBufferError) ;
        }

        Ok(())
    }

}
