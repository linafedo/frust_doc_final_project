use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc::Receiver;

type ReceiverJob = Arc<Mutex<Receiver<Job>>>;

// dyn - опред дин тип-ции для трейта
// Send - предача данных между потоками
// 'static
type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        // заранее выделяем определенный размер памяти, *немного эффективнее, чем ::new
        let mut workers = Vec::with_capacity(size);

        // создаем канал
        let (sender, receiver) = mpsc::channel();

        // оборачиваем ресивер в Arc для возможности использовать между потоками
        // оборачиваем в мьютекс для ограничения доступа к ресиверу по очереди
        let receiver = Arc::new(Mutex::new(receiver));

        for index in 0 .. size {
            // клонируем указатель на ресивер, чтобы исп-ть его в разных потоках
            let worker = Worker::new(index, Arc::clone(&receiver));
            workers.push(worker);
        }

        // передаем сендер в экземпляр для возможности отправить замыкание ресиверу через execute
        ThreadPool { workers, sender: Some(sender) }
    }

    pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static {
        let job = Box::new(f);
        // отправляем событие в ресивер
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: ReceiverJob) -> Worker {
        // создаем поток
        let thread = thread::spawn( move || loop {
            let message = receiver.lock()// блокируем  доступ других  потоков к ресиверу
                .unwrap()// разворачиваем result
                .recv();// блокируем текущий поток и ждем входящее значение в ресивер

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");
                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
            // после выполнения значения переменной job дропаются все значения, исп-мые в переменной
            // соотв-нно мьютекс разлачивается
         });

        Worker{ id, thread: Some(thread) }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join()// блокируем основной поток, пока не завершится этот
                    .unwrap();
            }
        }
    }
}