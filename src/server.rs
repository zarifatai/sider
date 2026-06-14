use crate::RESP;
use crate::connection::ConnectionMessage;
use crate::request::Request;
use crate::storage::Storage;
use tokio::sync::mpsc;

// pub fn process_request(request: Request, storage: Arc<Mutex<Storage>>) -> StorageResult<RESP> {
pub async fn process_request(request: Request, server: &mut Server) {
    let elements = match &request.value {
        RESP::Array(v) => v,
        _ => {
            panic!()
        }
    };

    let mut command = Vec::new();

    for elem in elements.iter() {
        match elem {
            RESP::BulkString(v) => command.push(v.clone()),
            _ => {
                panic!()
            }
        }
    }

    let storage = match server.storage.as_mut() {
        Some(storage) => storage,
        None => panic!(),
    };

    let _response = storage.process_command(&command);
}

pub struct Server {
    pub storage: Option<Storage>,
}

impl Server {
    pub fn new() -> Self {
        Self { storage: None }
    }

    pub fn set_storage(&mut self, storage: Storage) {
        self.storage = Some(storage);
    }
}

pub async fn run_server(mut server: Server, mut crx: mpsc::Receiver<ConnectionMessage>) {
    loop {
        tokio::select! {
            Some(message) = crx.recv() => {
                match message {
                    ConnectionMessage::Request(request) => {
                        process_request(request, &mut server).await
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_new() {
        let server: Server = Server::new();

        match server.storage {
            Some(_) => panic!(),
            None => (),
        };
    }

    #[test]
    fn test_set_storage() {
        let storage = Storage::new();

        let mut server: Server = Server::new();
        server.set_storage(storage);

        match server.storage {
            Some(_) => (),
            None => panic!(),
        };
    }
}
