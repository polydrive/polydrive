pub mod client {
    tonic::include_proto!("client");
}

pub mod file {
    use notify::DebouncedEvent;
    tonic::include_proto!("file");
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
}

pub mod upload {
    tonic::include_proto!("upload");
}

pub mod server {
    tonic::include_proto!("server");
}
