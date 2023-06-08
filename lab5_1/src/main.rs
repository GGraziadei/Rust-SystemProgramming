use std::env::Args;
use std::thread;
use std::thread::current;
use std::time::Duration;
use crate::thread_pool::ThreadPool;

mod worker;
mod scheduler;
mod thread_pool;



fn main() {
    // alloca i worker
    let mut threadpool = ThreadPool::new(10);

    for x in 0..100 {
        threadpool.execute(move || {

            fn factorial(num:i32)->i32 {
                if num == 1 {
                    return 1
                } else {
                    return num * factorial(num-1)
                }
            }

            println!("long running task {} => {} \n threadId: {:?} ", x, factorial(x), current().id());
            //thread::sleep(Duration::from_millis(1000))
        });
    }
    // just to keep the main thread alive
    loop {thread::sleep(Duration::from_millis(1000))};
}