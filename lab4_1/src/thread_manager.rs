use std::collections::HashMap;
use std::process::exit;
use std::sync::mpsc::{Receiver, SyncSender};
use std::thread::{Builder, current, ThreadId};

pub struct ThreadManager
{
    receivers : HashMap<ThreadId , (Receiver<bool> , SyncSender<bool>)>,
}

impl  ThreadManager {

    pub fn new() -> Self
    {
        Self{
            receivers: HashMap::<ThreadId, (Receiver<bool> , SyncSender<bool>)>::new()
        }
    }

    pub fn generate_threads(&mut self , n : usize ) -> ()
    {
        for i in 0..n
        {
            let (tx1, rx1) = std::sync::mpsc::sync_channel::<bool>(1);
            let (tx2, rx2) = std::sync::mpsc::sync_channel::<bool>(1);

            let tid = std::thread::spawn(move || {
                let mut cycle = 0;
                loop {
                    tx1.send(true);
                    let res = rx2.recv().unwrap();
                    assert!(res);

                    println!("T{:?} - cycle{}", current().id() , cycle);
                    cycle += 1;

                    if cycle == 10
                    {
                        exit(1);
                    }
                }
            } ).thread().id();
            self.receivers.insert(tid, (rx1, tx2));
        }
    }

    pub fn manage(&mut self) -> ()
    {
        loop {
            for t in self.receivers.values(){
                let res = t.0.recv().unwrap();
                assert!(res);
            }

            for t in self.receivers.values(){
                t.1.send(true);
            }

        }
    }

}