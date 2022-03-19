use notify::DebouncedEvent;

tonic::include_proto!("server");

impl From<&DebouncedEvent> for FileEventType {
    fn from(e: &DebouncedEvent) -> Self {
        match e {
            DebouncedEvent::Create(_) => FileEventType::Create,
            DebouncedEvent::Write(_) => FileEventType::Update,
            DebouncedEvent::Remove(_) => FileEventType::Delete,
            _ => FileEventType::Unknown,
        }
    }
}
