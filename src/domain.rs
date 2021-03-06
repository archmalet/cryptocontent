use std::collections::HashMap;
use uuid::Uuid;
use chrono::Date;
use chrono::DateTime;
use chrono::Local;
use chrono::Duration;

pub struct Account {
    pub items: Vec<Calendar>
}


/// This struct is used to store information about a single calendar,
/// including the events in it.
///
/// Events are stored in a HashMap, saved as Days containing a list of Events.
#[derive(Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Calendar {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub sync: bool,
    days: HashMap<String, Vec<Event>>,
}

/// An Event stores information about, you guessed it, an event in time. They are to be stored in
/// an instance of Calendar.
#[derive(Debug, PartialEq, Clone, RustcEncodable, RustcDecodable)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub location: String,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
    pub sync: bool,
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
/// Different types of entries.
pub enum EntryType {
    Create,
    Update,
    Delete
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
/// Representation of a single entry in an Eventlog.
pub struct EventLogEntry {
    pub id: String,
    pub entry_type: EntryType,
    pub obj_id: String,
    pub data: String
}

impl EventLogEntry {
    pub fn new(entry_type: EntryType, obj_id: &str, data: &str) -> EventLogEntry {
        EventLogEntry{
            id: Uuid::new_v4().to_string(),
            entry_type: entry_type,
            obj_id: obj_id.to_string(),
            data: data.to_string(),
        }
    }
}

/// In Calendar the Hashmaps uses DateTime<Local> as keys, because they have serde support. If
/// Date<Local> gets serde support, this should be used.
impl Calendar {

    /// Function to create a new Calendar struct with name and description.
    /// The sync bit is there to determine if this calendar is to be synced with online storage or
    /// not.
    pub fn new(name: &str, desc: &str, sync: bool) -> Calendar {
        Calendar {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            desc: desc.to_string(),
            sync: sync,
            days: HashMap::new(),
        }
    }

    /// Returns a vector of all the events currently stored in this instance of Calendar.
    pub fn get_events(&self) -> Vec<&Event> {
        let d = self.days.values();
        d.flat_map(|d| d.into_iter()).collect::<Vec<_>>()
    }

    /// Returns a slice of all the Events on the specified date. None if no event is saved for the
    /// given date.
    pub fn get_events_by_day(&self, date: Date<Local>) -> Option<&[Event]> {
        let date = date.and_hms(0, 0, 0).to_string();

        match self.days.get(&date) {
            Some(d) => Some(d),
            None => None,
        }
    }

    /// Stores an Event in the Calendar. If the date of the event isn't already a key in events
    /// hashmap the key is generated and event is saved in it's value list.
    /// TODO: WTF Ownership madness
    pub fn add_event(&mut self, e: Event) {
        let date = e.start.date().and_hms(0, 0, 0).to_string();

        if !(self.days.contains_key(&date)) {
            self.days.insert(date.clone().to_string(), Vec::new());
        }

        self.days.get_mut(&date).unwrap().push(e);
    }

    /// Deletes an Event in the Calendar. If the event is not found nothing happens.
    pub fn delete_event(&mut self, e: &Event) {
        let date = e.start.date().clone().and_hms(0, 0, 0).to_string();

        if !(self.days.contains_key(&date)) {
            return
        }

        let index = match self.days.get(&date).unwrap().iter().position(|x| x.id == e.id) {
            Some(i) => i,
            None => return,
        };

        self.days.get_mut(&date).unwrap().remove(index);
    }

    /// Repeats the event n times changing only the dates, with one week distance between them.
    pub fn repeat_event_n_times(&mut self, e: &Event, n: usize) {
        for _ in 0..n {
            let er = e.repeat(Duration::weeks(1));
            self.add_event(er);
        }
    }
}


impl Event {
    pub fn new(name: &str, desc: &str, location: &str) -> Event {
        Event {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            desc: desc.to_string(),
            location: location.to_string(),
            start: Local::now(),
            end: Local::now() + Duration::hours(1),
            sync: false,
        }
    }

    /// Repeats the event, returning the new instance, starting at given date and time. The
    /// difference between start and end date and time of the two events is the same.
    pub fn repeat(&self, distance: Duration) -> Event {

        Event {
            id: Uuid::new_v4().to_string(),
            name: self.name.clone(),
            desc: self.desc.clone(),
            location: self.location.clone(),
            start: self.start + distance,
            end: self.start + distance + (self.end - self.start),
            sync: false,
        }
    }
}
