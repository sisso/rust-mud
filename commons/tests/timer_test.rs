use std::collections::VecDeque;

struct Entry<T> {
    time: f64,
    value: T,
}

struct Timer<T> {
    current: f64,
    events: Vec<Entry<T>>,
}

impl <T> Timer<T> {
    pub fn new() -> Self {
        Timer { 
            current: 0.0, 
            events: Default::default(),
         }
    }

    pub fn schedule(&mut self, value: T, time: f64) {
        self.events.push(Entry { time, value: value } );
    }

    pub fn check(&mut self, total_time: f64) -> Vec<T> {
        assert!(self.current <= total_time);

        self.current = total_time;

        let mut indexes = VecDeque::new();
        for (i, e) in self.events.iter().enumerate() {
            if e.time <= total_time {
                indexes.push_front(i);
            }
        }

        let mut buffer = VecDeque::new();
        for i in indexes {
            let e = self.events.remove(i);
            buffer.push_front(e.value);
        }

        buffer.into()
   }
}

#[test]
pub fn test_timer() {
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
pub fn test_timer_fail_if_reverse_time() {
    let mut timer: Timer<u32> = Timer::new();
    timer.check(3.0);
    timer.check(1.0);
}

