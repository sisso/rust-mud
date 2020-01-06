use std::collections::{VecDeque, BTreeMap};
use std::cmp::Reverse;
use std::cmp::Ordering;

type EntryKey = (u64, u64);

#[derive(Debug, Clone)]
struct Entry<T> {
    id: u64,
    time: f64,
    value: T,
}

struct Timer<T> {
    index: u64,
    current: f64,
    events: BTreeMap<EntryKey, Entry<T>>,
}

impl <T> Timer<T> {
    pub fn new() -> Self {
        Timer {
            index: 0,
            current: 0.0, 
            events: Default::default(),
         }
    }

    pub fn schedule(&mut self, value: T, time: f64) {
        let next_index = self.index;
        self.index += 1;
        
        let key = (total_time_to_cap_time(time), next_index);
        self.events.insert(key, Entry { 
            id: next_index,
            time: time,
            value: value,
        });
    }

    pub fn check(&mut self, total_time: f64) -> Vec<T> {
        assert!(self.current <= total_time);

        self.current = total_time;

        let mut indexes = VecDeque::new();
        for (k, v) in self.events.iter() {
            if v.time <= total_time {
                indexes.push_front(k.clone());
            }
        }

        let mut buffer = VecDeque::new();
        for i in indexes {
            let e = self.events.remove(&i).unwrap();
            buffer.push_front(e.value);
        }

        buffer.into()
   }

    pub fn peek(&self) -> Option<f64> {
        self.events.keys().next().map(|(cap_time, _)| {
            cap_time_to_total_time(*cap_time)
        })
    }
}

#[test]
fn test_timer() {
    let mut timer: Timer<&str> = Timer::new();
    
    timer.schedule("Z", 0.0);
    timer.schedule("A", 2.0);
    timer.schedule("B", 3.5);
    timer.schedule("C", 4.0);
    timer.schedule("D", 10.0);

    // initial element
    let mut result = timer.check(0.0);
    assert_eq!(result.len(), 1);
    assert_eq!("Z", result.remove(0));

    // no new element
    let result = timer.check(1.0);
    assert_eq!(result.len(), 0);

    // normal case
    let mut result = timer.check(2.01);
    assert_eq!(result.len(), 1);
    assert_eq!("A", result.remove(0));

    // multiple case
    let mut result = timer.check(4.00);
    assert_eq!(result.len(), 2);
    assert_eq!("B", result.remove(0));
    assert_eq!("C", result.remove(0));

    // no new element
    let result = timer.check(6.0);
    assert_eq!(result.len(), 0);

    // huge jump
    let mut result = timer.check(1000.00);
    assert_eq!(result.len(), 1);
    assert_eq!("D", result.remove(0));

    // no new element
    let result = timer.check(10000.0);
    assert_eq!(result.len(), 0);
}

#[test]
#[should_panic]
fn test_timer_fail_if_reverse_time() {
    let mut timer: Timer<u32> = Timer::new();
    timer.check(3.0);
    timer.check(1.0);
}

#[test]
fn test_timer_peek_should_return_next_trigger() {
    let mut timer: Timer<&str> = Timer::new();
    assert!(timer.peek().is_none());

    timer.schedule("A", 2.0);
    timer.schedule("B", 10.0);

    assert_eq!(timer.peek(), Some(2.0));

    let _ = timer.check(2.5);
    assert_eq!(timer.peek(), Some(10.0));

    let _ = timer.check(20.0);
    assert!(timer.peek().is_none());
}

fn cap_time_to_total_time(cap_time: u64) -> f64 {
    (cap_time as f64) / 100.0
}

fn total_time_to_cap_time(total_time: f64) -> u64 {
    (total_time * 100.0) as u64
}

#[test]
fn test_to_cap_time() {
    assert_eq!(total_time_to_cap_time(0.0), 0);
    assert_eq!(total_time_to_cap_time(1.1), 110);
    assert_eq!(total_time_to_cap_time(2.22), 222);
    assert_eq!(total_time_to_cap_time(3.333), 333);
    assert_eq!(total_time_to_cap_time(444444.4444), 44444444);
}

#[test]
fn test_from_cap_time() {
    assert_eq!(cap_time_to_total_time(0), 0.0);
    assert_eq!(cap_time_to_total_time(110), 1.10);
}

