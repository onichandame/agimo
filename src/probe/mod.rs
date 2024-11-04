use std::collections::HashMap;

use k8s_openapi::api::apps::v1::{Deployment, StatefulSet};
use kube::{Api, Client};

use crate::event::{Event, Sender, };

pub(crate) async fn probe(sender: Sender) -> ! {
    let client = Client::try_default().await.unwrap();
    let mut subscriber = sender.subscribe();
    let host_to_workload_map=HashMap::new()
    loop {
        if let Ok(event) = subscriber.recv().await {
            match event {
                Event::GetReplicas { host, sender } => {
                    let ready_count = match host_to_workload_map.get(&host) {
                        Some(Workload { namespace, name, ty }) => {
                            match ty {
                                WorkloadType::Deployment => {
                                    Api::<Deployment>::namespaced(client.clone(), namespace)
                                        .get_status(name)
                                        .await
                                        .map(|res| res.status)
                                        .map(|stat| {
                                            stat.map(|stat| stat.ready_replicas.unwrap_or(0))
                                                .unwrap_or(0)
                                        })
                                        .unwrap_or(0)
                                }
                                WorkloadType::StatefulSet => {
                                    Api::<StatefulSet>::namespaced(client.clone(), namespace)
                                        .get_status(name)
                                        .await
                                        .map(|res| res.status)
                                        .map(|stat| {
                                            stat.map(|stat| stat.ready_replicas.unwrap_or(0))
                                                .unwrap_or(0)
                                        })
                                        .unwrap_or(0)
                                }
                            }
                        }
                        None => 0,
                    };
                    sender.send(ready_count.try_into().unwrap()).await.unwrap();
                }
            }
        }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Workload{
    namespace:String,
    name:String,
    ty:WorkloadType
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorkloadType {
    Deployment,
    StatefulSet,
}
