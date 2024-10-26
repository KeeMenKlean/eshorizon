use uuid::Uuid;

// Event type and aggregate type for demonstration purposes.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct EventType(Uuid);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct AggregateType(Uuid);

// Event trait with basic methods for EventType and AggregateType.
pub trait Event {
    fn event_type(&self) -> EventType;
    fn aggregate_type(&self) -> AggregateType;
}

// Trait for EventMatcher.
pub trait EventMatcher {
    fn matches(&self, event: &dyn Event) -> bool;
}

// MatchEvents matches any of the event types.
pub struct MatchEvents {
    event_types: Vec<EventType>,
}

impl MatchEvents {
    pub fn new(event_types: Vec<EventType>) -> Self {
        MatchEvents { event_types }
    }
}

impl EventMatcher for MatchEvents {
    fn matches(&self, event: &dyn Event) -> bool {
        self.event_types.iter().any(|&t| event.event_type() == t)
    }
}

// MatchAggregates matches any of the aggregate types.
pub struct MatchAggregates {
    aggregate_types: Vec<AggregateType>,
}

impl MatchAggregates {
    pub fn new(aggregate_types: Vec<AggregateType>) -> Self {
        MatchAggregates { aggregate_types }
    }
}

impl EventMatcher for MatchAggregates {
    fn matches(&self, event: &dyn Event) -> bool {
        self.aggregate_types.iter().any(|&t| event.aggregate_type() == t)
    }
}

// MatchAny matches any of the matchers.
pub struct MatchAny {
    matchers: Vec<Box<dyn EventMatcher>>,
}

impl MatchAny {
    pub fn new(matchers: Vec<Box<dyn EventMatcher>>) -> Self {
        MatchAny { matchers }
    }
}

impl EventMatcher for MatchAny {
    fn matches(&self, event: &dyn Event) -> bool {
        self.matchers.iter().any(|matcher| matcher.matches(event))
    }
}

// MatchAll matches all of the matchers.
pub struct MatchAll {
    matchers: Vec<Box<dyn EventMatcher>>,
}

impl MatchAll {
    pub fn new(matchers: Vec<Box<dyn EventMatcher>>) -> Self {
        MatchAll { matchers }
    }
}

impl EventMatcher for MatchAll {
    fn matches(&self, event: &dyn Event) -> bool {
        self.matchers.iter().all(|matcher| matcher.matches(event))
    }
}

// Sample Event struct for testing purposes.
#[derive(Debug)]
pub struct TestEvent {
    event_type: EventType,
    aggregate_type: AggregateType,
}

impl TestEvent {
    pub fn new(event_type: EventType, aggregate_type: AggregateType) -> Self {
        TestEvent {
            event_type,
            aggregate_type,
        }
    }
}

impl Event for TestEvent {
    fn event_type(&self) -> EventType {
        self.event_type
    }

    fn aggregate_type(&self) -> AggregateType {
        self.aggregate_type
    }
}

// Test cases for the matchers.
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_match_events() {
        let event_type1 = EventType(Uuid::new_v4());
        let event_type2 = EventType(Uuid::new_v4());
        let matcher = MatchEvents::new(vec![event_type1]);

        let event = TestEvent::new(event_type1, AggregateType(Uuid::new_v4()));
        assert!(matcher.matches(&event));

        let event = TestEvent::new(event_type2, AggregateType(Uuid::new_v4()));
        assert!(!matcher.matches(&event));
    }

    #[test]
    fn test_match_aggregates() {
        let aggregate_type1 = AggregateType(Uuid::new_v4());
        let aggregate_type2 = AggregateType(Uuid::new_v4());
        let matcher = MatchAggregates::new(vec![aggregate_type1]);

        let event = TestEvent::new(EventType(Uuid::new_v4()), aggregate_type1);
        assert!(matcher.matches(&event));

        let event = TestEvent::new(EventType(Uuid::new_v4()), aggregate_type2);
        assert!(!matcher.matches(&event));
    }

    #[test]
    fn test_match_any() {
        let event_type1 = EventType(Uuid::new_v4());
        let aggregate_type1 = AggregateType(Uuid::new_v4());

        let matcher1 = Box::new(MatchEvents::new(vec![event_type1]));
        let matcher2 = Box::new(MatchAggregates::new(vec![aggregate_type1]));

        let matcher_any = MatchAny::new(vec![matcher1, matcher2]);

        let event = TestEvent::new(event_type1, aggregate_type1);
        assert!(matcher_any.matches(&event));
    }

    #[test]
    fn test_match_all() {
        let event_type1 = EventType(Uuid::new_v4());
        let aggregate_type1 = AggregateType(Uuid::new_v4());

        let matcher1 = Box::new(MatchEvents::new(vec![event_type1]));
        let matcher2 = Box::new(MatchAggregates::new(vec![aggregate_type1]));

        let matcher_all = MatchAll::new(vec![matcher1, matcher2]);

        let event = TestEvent::new(event_type1, aggregate_type1);
        assert!(matcher_all.matches(&event));

        let event = TestEvent::new(EventType(Uuid::new_v4()), aggregate_type1);
        assert!(!matcher_all.matches(&event));
    }
}