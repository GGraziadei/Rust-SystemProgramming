use std::collections::{HashMap, LinkedList, VecDeque};
use std::thread::current;
use crossbeam::channel::{Receiver, Sender};
use crossbeam::select;
use crate::worker::{Status, Worker, WorkerMessage};
use crate::worker::Status::{READY, RUNNING, WAITING};
use crate::worker::WorkerMessage::{ JobCompleted, NewJob, JobAccepted};

pub struct Scheduler<F> {
    rx_pool : Receiver<WorkerMessage<F>>,
    rx_workers : Receiver<WorkerMessage<F>>,
    jobs_queue : VecDeque<F>,
    workers : HashMap<Status, VecDeque< WorkerScheduler<F> >>,
}

pub struct GenericErr;

struct WorkerScheduler<F> {
    id : usize,
    tx : Sender<WorkerMessage<F>>
}

impl<F> WorkerScheduler<F>
where F : FnOnce()->() + Send + 'static
{
    pub fn new(id : usize, tx : Sender<WorkerMessage<F>>)
        ->Self
    {
        Self{
            id,
            tx
        }
    }
}

impl <F> Scheduler<F>
where F : FnOnce()->() + Send + 'static
{

    pub fn new (n : usize, rx_pool : Receiver<WorkerMessage<F>> )
        ->(Self, Vec<Worker<F>>)
    {

        let mut workers = HashMap::<Status, VecDeque<WorkerScheduler<F>> >::new();

        workers.insert(RUNNING, Default::default());
        workers.insert(READY, Default::default());
        workers.insert(WAITING, Default::default());

        let mut workers_res = Vec::<Worker<F>>::with_capacity(n);
        let (tx_broadcast, rx_broadcast ) = crossbeam::channel::bounded::<WorkerMessage<F>>(2 * n + 1);

        for id in 0..n
        {
            let (tx,rx) = crossbeam::channel::bounded::<WorkerMessage<F>>(n * 2 +1);
            let worker = Worker::<F>::new(id.clone(), rx, tx_broadcast.clone() );
            let entry = WorkerScheduler::<F>::new(id.clone(), tx);
            workers_res.push(worker);
            workers.entry(READY)
                .and_modify(|v| {v.push_front(entry)});
        }

        (Self{
            rx_pool,
            rx_workers : rx_broadcast,
            jobs_queue: VecDeque::<F>::new(),
            workers
        },workers_res)

    }

    pub fn run(&mut self)
    {
        loop {

            select! {
                recv(self.rx_pool) -> msg => {

                    /*
                        - ThreadPool send new work
                    */

                    if let Ok(NewJob(mut f)) = msg {
                        if self.workers.get(&READY).unwrap().is_empty(){
                            self.jobs_queue.push_front(f);
                        }else{
                            let worker = self.workers.get_mut(&READY)
                                .expect("Error: worker list required")
                                .pop_back().expect("Error: worker required");

                            //Send message to worker
                            worker.tx.send(NewJob(f)).expect("Send message");

                            self.workers.get_mut(&WAITING)
                                .expect("Error: worker list required")
                                .push_front(worker);

                        }
                    }
                },
                recv(self.rx_workers) -> msg => {

                    /*
                        - Worker accepts work
                        - Worker ends work
                    */

                    if let Ok(JobAccepted(id)) = msg {
                        let worker_id = self.workers.get(&WAITING)
                            .expect("Error: worker list required")
                            .iter().position(|w| {w.id == id})
                            .expect("Error: worker required");

                        let worker = self.workers.get_mut(&WAITING)
                            .expect("Error: worker list required")
                            .remove(worker_id)
                            .expect("Error: worker required");

                        self.workers.get_mut(&RUNNING)
                            .expect("Error: worker list required")
                            .push_front(worker);

                    }else if let Ok(JobCompleted(id)) = msg{

                        let worker_id = self.workers.get(&RUNNING)
                            .expect("Error: worker list required")
                            .iter().position(|w| {w.id == id})
                            .expect("Error: worker required");

                        let worker = self.workers.get_mut(&RUNNING)
                            .expect("Error: worker list required")
                            .remove(worker_id)
                            .expect("Error: worker required");

                        self.workers.get_mut(&READY)
                            .expect("Error: worker list required")
                            .push_front(worker);

                    }

                    while ! self.workers.get(&READY).expect("Error: Ready workers required").is_empty()
                            && ! self.jobs_queue.is_empty(){

                        let worker = self.workers.get_mut(&READY)
                                .expect("Error: worker list required")
                                .pop_back().expect("Error: worker required");

                            //Send message to worker
                        worker.tx.send(NewJob(self.jobs_queue.pop_back().unwrap()))
                            .expect("Send message");

                        self.workers.get_mut(&WAITING)
                            .expect("Error: worker list required")
                            .push_front(worker);
                    }

                }
            }
        }
    }




}