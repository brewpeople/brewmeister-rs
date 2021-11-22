use anyhow::{anyhow, Result};
use rand::Rng;
use structopt::StructOpt;

fn parse_temperature(src: &str) -> Result<f32> {
    let temperature = src.parse::<f32>()?;

    if temperature < 20.0 || temperature > 99.0 {
        Err(anyhow!("Temperature must be between [20, 99]"))
    }
    else {
        Ok(temperature)
    }
}

#[derive(StructOpt)]
enum Opt {
    StressTest {},
    Read {},
    SetTemperature {
        #[structopt(long, parse(try_from_str = parse_temperature))]
        target: f32,
    },
}

async fn stress_test(client: comm::Comm) -> Result<()> {
    let mut rng = rand::thread_rng();

    let num_iterations = 100;
    let mut num_fails = 0;
    let bar = indicatif::ProgressBar::new(num_iterations);

    bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {msg}")
            .progress_chars("#:."),
    );

    for _ in 0..num_iterations {
        let expected = rng.gen_range(20.0..100.0);
        client.set_temperature(expected).await?;
        let state = client.read_state().await?;

        if (expected - state.temperature).abs() >= f32::EPSILON {
            bar.set_message("failed to r/w temperature");
            num_fails += 1;
        }

        let expected = rng.gen_bool(0.5);
        client.write_stirrer(expected).await?;
        let state = client.read_state().await?;

        if state.stirrer_on != expected {
            bar.set_message("failed to r/w stirrer");
            num_fails += 1;
        }

        bar.inc(1);
    }

    bar.finish();
    println!(
        "{}/{} successful r/w operations",
        num_iterations * 2 - num_fails,
        num_iterations * 2
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let opts = Opt::from_args();
    let client = comm::Comm::new()?;

    match opts {
        Opt::StressTest {} => {
            stress_test(client).await?;
        }
        Opt::Read {} => {
            println!("{:#?}", client.read_state().await?);
        }
        Opt::SetTemperature { target } => {
            client.set_temperature(target).await?;
        }
    };

    Ok(())
}
