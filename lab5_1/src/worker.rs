use std::thread::current;
use crossbeam::channel::{Receiver, Sender};
use crate::worker::Status::{READY, RUNNING};
use crate::worker::WorkerMessage::NewJob;

#[derive(Hash, Copy, Clone, Eq, PartialEq)]
pub enum Status{
    READY,
    RUNNING,
    WAITING
}

pub enum  WorkerMessage<F>
{
    NewJob(F),
    JobAccepted(usize),
    JobCompleted(usize)
}

pub struct Worker<F>{
    id : usize,
    status : Status,
    rx : Receiver<WorkerMessage<F>>,
    tx : Sender<WorkerMessage<F>>
}

impl<F> Worker<F>
where F : FnOnce()->() + Send + 'static
{
    pub fn new(id : usize, rx : Receiver<WorkerMessage<F>>, tx :  Sender<WorkerMessage<F>>)
        -> Self
    {

        Self{
            id,
            status: READY,
            rx,
            tx
        }

    }

    pub fn run(&mut self) -> ()
    {
        loop {
            self.status = READY;

            //Sync - wait that scheduler send a new job
            // if new job is scheduled then execute it
            let job = self.rx.recv()
                .expect("Error during ack job reception.");

            if let NewJob( f ) = job
            {
                self.tx.send(WorkerMessage::JobAccepted(self.id.clone()))
                    .expect("Error during job acceptance");
                self.status = RUNNING;

                f(); /*Execute the function*/

                self.tx.send(WorkerMessage::JobCompleted(self.id.clone()))
                    .expect("Error during ack job completed.")

            }
        }

    }

}