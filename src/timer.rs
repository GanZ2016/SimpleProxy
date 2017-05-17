use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::thread;

//Create a concurrent timer to let tunnel thread 
//waiting for a certain amount of time.

pub struct Timer;

impl Timer {
    pub fn new(ms: u32) -> Receiver<()> {
        let (tx, rx) = channel();

        thread::spawn(move || {
            timer_loop(tx, ms);
        });

        rx
    }
}

fn timer_loop(tx: Sender<()>, ms: u32) {
    let t = Duration::from_millis(ms as u64);
    loop {
        thread::sleep(t);
        match tx.send(()) {
            Ok(_) => {},
            Err(_) => break
        }
    }
}
#[test]

fn test_timer_work_properly(){
    let current_time = time::get_time();
    let timer = Timer::new(500);
    let duration = time::get_time() - current_time;
    assert_eq!(500, duration.num_milliseconds());
}