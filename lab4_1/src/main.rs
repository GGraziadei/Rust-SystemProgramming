use std::fmt::format;
use std::sync::Arc;
use std::time::Duration;
use crate::channel_barrier::ChannelBarrier;
use crate::cyclic_barrier::CyclicBarrier;
use crate::thread_manager::ThreadManager;

mod cyclic_barrier;
mod channel_barrier;
mod thread_manager;

const BARRIER_SIZE : usize = 3;

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::{BARRIER_SIZE, ChannelBarrier};
    use crate::cyclic_barrier::CyclicBarrier;

    #[test]
    fn barrier_mutex() {
        let mut context = Arc::new(CyclicBarrier::<usize>::new(BARRIER_SIZE , BARRIER_SIZE));
        std::thread::scope(|s| {
            for i in 0..BARRIER_SIZE {
                let context_ti = context.clone();
                s.spawn(move || {
                    for cycle in 0..10 {
                        context_ti.wait();
                        println!("T{} - SC cycle{}" , i, cycle);
                    }
                });
            }
        });
    }

    #[test]
    fn barrier_channels() {
        let mut contexts : Vec<ChannelBarrier> = Vec::<ChannelBarrier>::with_capacity(BARRIER_SIZE);

        for i in 0..BARRIER_SIZE
        {
            let (tx, rx) = std::sync::mpsc::sync_channel::<bool>(BARRIER_SIZE);
            contexts.push(ChannelBarrier::new(rx, tx));
        }

        for i in 0..contexts.len() {
            for j in 0..contexts.len() {
                if i != j {
                    let tid = & (i as u8);
                    let sender = contexts[i].get_sender().expect("Impossibile accedere al sender per thread " );
                    contexts[j].add_sender( sender , tid );
                }
            }
        }

        std::thread::scope(|s| {
            let mut tid_mut = 0;
            for context in contexts.iter_mut() {
                let tid = tid_mut;
                s.spawn(move || {
                    for cycle in 0..10 {
                        context.wait();
                        println!("T{} - SC cycle{}" , tid,  cycle);
                    }
                });
                tid_mut += 1;
            }
        });
    }
}


fn main() {

    let mut t_manager = ThreadManager::new();
    t_manager.generate_threads(BARRIER_SIZE);
    t_manager.manage();
}
