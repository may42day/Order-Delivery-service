use delivery_order::utils::configs::{Application, Config, GrpcServer};
use std::fmt::{Debug, Display};
use tokio::task::JoinError;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = Config::init().await;

    let application = Application::build(&config).await?;
    let grpc_server = GrpcServer::build().await?;

    let application_task = tokio::spawn(application.run_untill_stopped());
    let grpc_server_task = tokio::spawn(grpc_server.run_untill_stopped(config.clone()));

    tokio::select! {
        task = application_task => report_exit("Application", task),
        task = grpc_server_task =>  report_exit("gRPC Server", task),
    };
    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{}' task failed to complete",
                task_name
            )
        }
    }
}
