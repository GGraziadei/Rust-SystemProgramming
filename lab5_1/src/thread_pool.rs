use std::thread::{JoinHandle, ThreadId};
use crossbeam::channel::Sender;
use crate::scheduler::Scheduler;
use crate::worker::{Worker, WorkerMessage};
use crate::worker::WorkerMessage::NewJob;

pub struct ThreadPool<F> {
    scheduler_tid : ThreadId,
    workers_tid : Vec<ThreadId>,
    scheduler_tx : Sender<WorkerMessage<F>>
}

impl<F> ThreadPool<F>
where F : FnOnce()->() + Send + 'static
{
    pub fn new(n : usize)
        -> Self
    {
        let (scheduler_tx, scheduler_rx) = crossbeam::channel::unbounded::<WorkerMessage<F>>();
        let (mut scheduler, mut workers) = Scheduler::new(n, scheduler_rx);

        /*Generate scheduler thread */
        let scheduler = std::thread::spawn(move || {
            scheduler.run();
        });
        let scheduler_tid = scheduler.thread().id();

        /*Generate workers threads*/
        let mut workers_tid = Vec::<ThreadId>::with_capacity(n);
        let mut workers_handle = Vec::with_capacity(n);

        for mut worker in workers.into_iter() {

            let worker_handle = std::thread::spawn(move ||{
                worker.run();
            });

            workers_tid.push(worker_handle.thread().id().clone());
            workers_handle.push(worker_handle);
        }

        Self{
            scheduler_tid,
            workers_tid,
            scheduler_tx
        }

    }

    pub fn execute(&mut self, job : F) -> ()
    {
        self.scheduler_tx.send(NewJob(job))
            .expect("Error during job transmission");
    }

}
