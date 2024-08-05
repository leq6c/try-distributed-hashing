use tonic::{transport::Server, Request, Response, Status};
use task_service::task_service_server::{TaskService, TaskServiceServer};
use task_service::{TaskRequest, TaskResponse};
use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::thread;

pub mod task_service {
    tonic::include_proto!("task");
}

#[derive(Debug, Default)]
pub struct MyTaskService {}

#[tonic::async_trait]
impl TaskService for MyTaskService {
    async fn run_task(
        &self,
        request: Request<TaskRequest>,
    ) -> Result<Response<TaskResponse>, Status> {
        let req = request.into_inner();
        let challenge: Arc<[u8; 32]> = Arc::new(req.challenge.try_into().expect("challenge must be 32 bytes"));
        let total_threads = req.total_threads as usize;
        let test_duration = req.test_duration;
        
        let mut handles = Vec::with_capacity(total_threads);

        for i in 0..total_threads {
            let challenge = Arc::clone(&challenge);
            handles.push(thread::spawn(move || {
                let timer = Instant::now();
                let first_nonce = u64::MAX.saturating_div(total_threads as u64).saturating_mul(i as u64);
                let mut nonce = first_nonce;
                loop {
                    let _hx = drillx::hash(&challenge, &nonce.to_le_bytes());

                    nonce += 1;

                    if timer.elapsed().as_secs() as i64 >= test_duration {
                        break;
                    }
                }

                nonce - first_nonce
            }));
        }

        let mut total_nonces = 0;
        for handle in handles {
            total_nonces += handle.join().unwrap();
        }

        let response = TaskResponse {
            nonce_count: total_nonces as i64,
        };

        Ok(Response::new(response))
    }
}

pub async fn run(addr: String) -> Result<(), Box<dyn std::error::Error>> {
    print!("Listening on: {}", addr);
    let addr = addr.parse()?;
    let task_service = MyTaskService::default();

    Server::builder()
        .add_service(TaskServiceServer::new(task_service))
        .serve(addr)
        .await?;

    Ok(())
}