//! Executes a brew "program", i.e. set target temperatures and wait until they are reached and
//! then wait more until the required duration has passed.

use crate::{devices, Result};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{sleep, Duration};
use tracing::{info, instrument, warn};

/// Used by the caller to get a result back from a command.
type Responder<T> = oneshot::Sender<Result<T>>;

/// Commands to send to the program channel.
pub enum Command {
    Start {
        id: i64,
        steps: Vec<models::Step>,
        resp: Responder<()>,
    },
}

/// Type alias for the command sender.
pub type Sender = mpsc::Sender<Command>;

#[instrument(skip(tx))]
async fn set_temperature(tx: devices::Sender, temperature: f32) -> Result<()> {
    let (resp, rx) = oneshot::channel();
    let command = devices::Command::SetTemperature { temperature, resp };
    let _ = tx.send(command).await;
    rx.await?
}

#[instrument(skip(tx))]
async fn read_temperature(tx: devices::Sender) -> Result<Option<f32>> {
    let (resp, rx) = oneshot::channel();
    let command = devices::Command::Read { resp };
    let _ = tx.send(command).await;
    Ok(rx.await??.current_temperature)
}

#[instrument(skip(tx))]
async fn wait_for(
    id: i64,
    tx: devices::Sender,
    temperature: f32,
    db: crate::db::Database,
) -> Result<()> {
    loop {
        let cloned = tx.clone();

        match read_temperature(cloned).await? {
            Some(current) => {
                db.add_sample(id, current).await?;

                if (current - temperature).abs() < 0.5 {
                    info!("Reached {:.2}C", current);
                    break;
                }
            }
            None => {
                // TODO: return after a few tries.
                warn!("No temperature received from the device");
            }
        }

        sleep(Duration::from_secs(5)).await;
    }

    Ok(())
}

/// Run the given program `steps` until completion.
#[instrument(skip_all)]
async fn run_program(
    id: i64,
    tx: devices::Sender,
    steps: Vec<models::Step>,
    db: crate::db::Database,
) -> Result<()> {
    for step in steps {
        info!(
            "Set target temperature to {}C and wait",
            step.target_temperature
        );
        set_temperature(tx.clone(), step.target_temperature).await?;
        wait_for(id, tx.clone(), step.target_temperature, db.clone()).await?;

        info!("Target temperature reached, waiting {:?}", step.duration);
        sleep(step.duration).await;
    }

    Ok(())
}

/// Run handler task receiving brew commands via `rx` and use `tx` to send device commands.
#[instrument(skip_all)]
pub async fn run(
    tx: devices::Sender,
    mut rx: mpsc::Receiver<Command>,
    db: crate::db::Database,
) -> Result<()> {
    while let Some(command) = rx.recv().await {
        let cloned = tx.clone();

        match command {
            Command::Start { id, steps, resp } => {
                // TODO: check if we are already running a brew program.
                let _ = resp.send(run_program(id, cloned, steps, db.clone()).await);
            }
        }
    }

    Ok(())
}