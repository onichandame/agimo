use base::Counter;
use memory::MemoryCounter;

use crate::event::{Event, Sender};

mod base;
mod memory;

pub(crate) async fn counter(sender: Sender) {
    let counter = MemoryCounter::new();
    let mut receiver = sender.subscribe();
    loop {
        if let Ok(event) = receiver.recv().await {
            match event {
                Event::InitRequest { host } => {
                    counter.inc(&host).await.unwrap();
                    sender
                        .send(Event::UpdateActive {
                            host: host.clone(),
                            active: counter.get_active(&host).await.unwrap(),
                        })
                        .unwrap();
                }
                Event::CloseRequest { host } => {
                    counter.dec(&host).await.unwrap();
                    sender
                        .send(Event::UpdateActive {
                            host: host.clone(),
                            active: counter.get_active(&host).await.unwrap(),
                        })
                        .unwrap();
                }
                Event::GetActive { host, sender } => {
                    let count = counter.get_active(&host).await.unwrap();
                    sender.send(count).await.unwrap();
                }
                _other => {}
            }
        }
    }
}
