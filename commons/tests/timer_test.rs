use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;

type Index = u64;
type Time = u64;
type EntryKey = (Reverse<Time>, Index);

struct Timer<T> {
    next_index: Index,
    current: Time,
    queue: BinaryHeap<EntryKey>,
    entries: HashMap<Index, T>,
}

impl <T> Timer<T> {
    pub fn new() -> Self {
        Timer {
            next_index: 0,
            current: 0,
            queue: Default::default(),
            entries: Default::default(),
         }
    }

    pub fn schedule(&mut self, value: T, total_time: f64) {
        let entry_index = self.next_index;
        self.next_index += 1;

        self.entries.insert(entry_index, value);

        let time = time_f64_to_time_u64(total_time);
        self.queue.push((Reverse(time), entry_index));
    }

    pub fn check(&mut self, total_time: f64) -> Vec<T> {
        let total_time = time_f64_to_time_u64(total_time);
        assert!(self.current <= total_time);
        self.current = total_time;

        let mut result = Vec::new();
        loop {
            match self.queue.peek() {
                Some((Reverse(time), id)) if time <= &total_time => {
                    let obj = self.entries.remove(id).unwrap();
                    result.push(obj);
                    self.queue.pop();
                },
                _ => break,
            }
        }

        result
   }

    pub fn peek(&self) -> Option<f64> {
        self.queue.peek().iter().next().map(|(reversed_time, _)| {
            time_u64_to_time_f64(reversed_time.0)
        })
    }
}

fn time_u64_to_time_f64(cap_time: u64) -> f64 {
    (cap_time as f64) / 100.0
}

fn time_f64_to_time_u64(total_time: f64) -> u64 {
    (total_time * 100.0) as u64
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

#[test]
fn test_to_cap_time() {
    assert_eq!(time_f64_to_time_u64(0.0), 0);
    assert_eq!(time_f64_to_time_u64(1.1), 110);
    assert_eq!(time_f64_to_time_u64(2.22), 222);
    assert_eq!(time_f64_to_time_u64(3.333), 333);
    assert_eq!(time_f64_to_time_u64(444444.4444), 44444444);
}

#[test]
fn test_from_cap_time() {
    assert_eq!(time_u64_to_time_f64(0), 0.0);
    assert_eq!(time_u64_to_time_f64(110), 1.10);
}
