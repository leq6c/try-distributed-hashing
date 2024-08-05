use task_service::task_service_client::TaskServiceClient;
use task_service::TaskRequest;
use std::sync::{Arc, Mutex};
use std::env;

pub mod task_service {
    tonic::include_proto!("task");
}

pub async fn run_client() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        eprintln!("Usage: {} client <threads> <test_duration> <server_addresses>", args[0]);
        eprintln!("Example: {} client 4 30 http://localhost:50051,http://localhost:50052", args[0]);
        return Ok(());
    }

    let threads: usize = args[2].parse().expect("cannot parse threads");
    let test_duration: i64 = args[3].parse().expect("cannot parse duration");
    let server_addresses: Vec<&str> = args[4].split(',').collect();

    let challenge = [0; 32];
    let client_handles = Arc::new(Mutex::new(Vec::new()));

    for (i, address) in server_addresses.iter().enumerate() {
        println!("Connecting to server: {}", address);
        let challenge = challenge.clone();
        let client_handles = Arc::clone(&client_handles);
        let address = address.to_string();
        tokio::spawn(async move {
            let mut client = TaskServiceClient::connect(address).await.unwrap();
            let request = tonic::Request::new(TaskRequest {
                thread_id: i as i32,
                total_threads: threads as i32,
                challenge: challenge.to_vec(),
                test_duration: test_duration,
            });
            let response = client.run_task(request).await.unwrap().into_inner();

            client_handles.lock().unwrap().push(response.nonce_count);
        });
    }

    loop {
        if Arc::strong_count(&client_handles) == 1 {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let handles = Arc::try_unwrap(client_handles).unwrap().into_inner().unwrap();
    let total_nonces: i64 = handles.iter().sum();
    let hashpower = total_nonces.saturating_div(test_duration);

    println!("Total nonces: {}", total_nonces);
    println!("{} H/s", hashpower);

    Ok(())
}
