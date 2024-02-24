use std::{
    sync::{mpsc::{self, Receiver}, Arc, Mutex}, 
    thread
};

// 工作线程
struct Worker {
    id : usize,                         // 线程id，线程唯一标识
    thread : thread::JoinHandle<()>,    // 线程句柄，用于控制和管理线程的生命周期
}

/*
这里定义了一个 Job 类型的别名，它是一个指向实现了 FnOnce() 特质的闭包函数的 Box，可以被发送到另一个线程执行。
Send 标记表示 Job 类型是可以安全地在多个线程间发送的。
'static 生命周期表示 Job 类型中的闭包函数不持有任何引用，即其生命周期为静态。
*/
type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    // 创建Worker实例
    // id线程唯一标识，receiver表示一个共享接受者，用于主线程获取工作任务
    fn new (id : usize, receiver : Arc<Mutex<mpsc::Receiver<Job>>>) ->Worker {
        // 循环创建一个线程
        let thread = thread::spawn(move || loop {
            // 获取了接收者的互斥锁，确保同时只有一个线程能够访问接收者。
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {id} got a job; executing.");
  
            job();
        });
        // 返回工作线程
        Worker { id, thread }
    }
}


// 构建线程池，用于管理和执行多个任务
pub struct ThreadPool {
    workers : Vec<Worker>,                  // 线程池中工作的线程集合
    sender : mpsc::Sender<Job>,             // 线程池的发送者
}

impl ThreadPool {
    // 创建ThreadPool实例
    pub fn new (size : usize) -> ThreadPool{
        assert!(size > 0);
        // 创建一个消息通道，返回发送者和接受者，用于在线程池和工作线程之间发送任务
        let (sender, receiver) = mpsc::channel();
        // 接受者被包装在Arc<Mutex<>>中，以便在线程间共享
        let receiver = Arc::new(Mutex::new(receiver));
        // 创建了一个容量为 size 的空 Vec
        let mut workers = Vec::with_capacity(size);
		// 循环创建了 size 个工作线程，并将它们添加到线程池中。
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        // 返回创建的线程池实例
        ThreadPool {workers, sender}
    }


    // 定义了一个 execute 方法，用于向线程池中提交任务
    // 该方法接受一个闭包函数 f 作为参数，并将其包装成一个 Job 类型的 Box，然后通过消息通道的发送者将任务发送给工作线程。
    pub fn execute<F>(&self, f: F)
    where 
        F : FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }

}