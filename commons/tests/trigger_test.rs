use std::collections::HashMap;

pub struct Listener {
    id: u32,
    kind: u32,
}

pub struct Trigger<T> {
    next_listener: u32,
    /// index is even type, key is listener id, value is position in events queue
    listeners_per_kind: HashMap<u32, HashMap<u32, usize>>,
    /// index is event type, value is published events
    events: HashMap<u32, Vec<T>>,
}

impl <T> Trigger<T> {
    pub fn new() -> Self {
        Trigger {
            next_listener: 0,
            listeners_per_kind: HashMap::new(),
            events: HashMap::new(),
        }
    }

    pub fn register(&mut self, event_kind: u32) -> Listener {
        let next = self.next_listener;
        self.next_listener += 1;

        let listeners = self.listeners_per_kind.entry(event_kind)
            .or_default();

        listeners.insert(next, 0);

        Listener {
            id: next,
            kind: event_kind,
        }
    }

    pub fn push(&mut self, event_kind: u32, event: T) {
        let events = match self.events.get_mut(&event_kind) {
            Some(events) => events,
            None if self.listeners_per_kind.contains_key(&event_kind) => {
                self.events.insert(event_kind, Vec::new());
                self.events.get_mut(&event_kind).unwrap()
            },
            None => return,
        };

        events.push(event);
    }

    pub fn take(&mut self, listener: &Listener) -> Vec<&T> {
        let index = self.listeners_per_kind.get_mut(&listener.kind)
            .unwrap()
            .get_mut(&listener.id)
            .unwrap();

        match self.events.get(&listener.kind) {
            Some(vec) => {
                let current = *index;
                *index = vec.len();
                vec.iter().skip(current).collect()
            },
            None => Vec::new(),
        }
    }

    pub fn len(&self, event_kind: u32) -> usize {
        self.events.get(&event_kind)
            .map(|vec| vec.len())
            .unwrap_or(0)
    }

    pub fn gc(&mut self) {
        let min_index_per_kind = self.listeners_per_kind.iter()
            .map(|(&kind, listeners)| {
                let min = listeners.iter()
                    .map(|(_, &pos)| pos)
                    .min()
                    .unwrap_or(0);

                (kind, min)
            }).collect::<Vec<_>>();

        for (kind, min_index) in min_index_per_kind {
            // update events
            let events = self.events.get_mut(&kind).unwrap();
            events.drain(0..min_index);

            // update indexes
            self.listeners_per_kind.get_mut(&kind)
                .unwrap()
                .iter_mut()
                .for_each(|(_, pos)| {
                    *pos -= min_index;
                })
        }
    }
}

#[test]
fn test_listeners() {
    let mut trigger = Trigger::new();

    let listener_0 = trigger.register(0);
    let listener_1 = trigger.register(0);
    let listener_2 = trigger.register(1);

    trigger.push(0, 0);
    trigger.push(0, 1);

    // take first events
    let result = trigger.take(&listener_0);
    assert_eq!(2, result.len());
    assert_eq!(0, *result[0]);
    assert_eq!(1, *result[1]);

    let result = trigger.take(&listener_1);
    assert_eq!(2, result.len());
    assert_eq!(0, *result[0]);
    assert_eq!(1, *result[1]);

    let result = trigger.take(&listener_2);
    assert_eq!(0, result.len());

    // second time is empty
    let result = trigger.take(&listener_0);
    assert_eq!(0, result.len());

    // noch einmal
    trigger.push(1, 2);

    let result = trigger.take(&listener_0);
    assert_eq!(0, result.len());

    let result = trigger.take(&listener_1);
    assert_eq!(0, result.len());

    let result = trigger.take(&listener_2);
    assert_eq!(1, result.len());
    assert_eq!(2, *result[0]);
}

#[test]
fn test_events_garbage_collect() {
    let mut trigger = Trigger::new();

    let listener_0 = trigger.register(0);
    let listener_1 = trigger.register(0);

    for i in 0..100 {
        trigger.push(0, i);
    }

    let result = trigger.take(&listener_0);
    assert_eq!(100, result.len());

    let result = trigger.take(&listener_1);
    assert_eq!(100, result.len());

    for i in 0..10 {
        trigger.push(0, i);
    }
    assert_eq!(110, trigger.len(0));

    trigger.gc();
    assert_eq!(10, trigger.len(0));

    let result = trigger.take(&listener_0);
    assert_eq!(10, result.len());

    let result = trigger.take(&listener_1);
    assert_eq!(10, result.len());

    trigger.gc();
    assert_eq!(0, trigger.len(0));
}

#[test]
fn test_trigger_should_not_store_events_without_listener() {
    let mut trigger = Trigger::new();

    for i in 0..100 {
        trigger.push(0, i);
    }

    assert_eq!(0, trigger.len(0));
}
