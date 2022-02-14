//! Serial communication with the Brewslave.

use byteorder::{ByteOrder, LittleEndian};
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Received NACK")]
    Nack,
    #[error("Received unexpected data")]
    UnexpectedData,
    #[error("Tokio I/O error")]
    TokioIo(#[from] tokio::io::Error),
    #[error("Tokio Serial error")]
    TokioSerial(#[from] tokio_serial::Error),
    #[error("Serial I/O timeout")]
    Timeout(#[from] tokio::time::error::Elapsed),
}

/// Serial communication structure wrapping the Brewslave protocol.
#[derive(Debug)]
pub struct Comm {
    stream: Arc<RwLock<SerialStream>>,
}

/// Current state of the Brewslave.
#[derive(Clone, Debug)]
pub struct State {
    pub current_temperature: Option<f32>,
    pub target_temperature: Option<f32>,
    pub stirrer_on: bool,
    pub heater_on: bool,
}

const RESPONSE_ACK: u8 = 0x80;
const RESPONSE_NACK: u8 = 0x40;
const RESPONSE_STIRRER_BIT: u8 = 0x1;
const RESPONSE_HEATER_BIT: u8 = 0x2;

enum Command {
    ReadState = 0x1,
    SetTemperature = 0x2,
    TurnStirrerOn = 0x3,
    TurnStirrerOff = 0x4,
}

fn ack_byte_to(ack: u8) -> Result<(), Error> {
    if (ack & RESPONSE_ACK) != 0 {
        Ok(())
    } else if (ack & RESPONSE_NACK) != 0 {
        Err(Error::Nack)
    } else {
        Err(Error::UnexpectedData)
    }
}

impl Comm {
    /// Create a new communication structure.
    ///
    /// As of now, it tries to open `/dev/tty/ACM0`.
    pub fn new(path: &Path) -> Result<Self, Error> {
        let stream = tokio_serial::new(path.to_string_lossy(), 115200)
            .flow_control(tokio_serial::FlowControl::None)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::One)
            .open_native_async()?;

        Ok(Self {
            stream: Arc::new(RwLock::new(stream)),
        })
    }

    /// Read the current state comprised of temperature and device states.
    pub async fn read_state(&self) -> Result<State, Error> {
        let mut stream = self.stream.write().await;
        stream.write_u8(Command::ReadState as u8).await?;

        let mut current: [u8; 4] = [0; 4];
        let mut target: [u8; 4] = [0; 4];
        let mut state: [u8; 1] = [0; 1];

        timeout(Duration::from_secs(1), stream.read_exact(&mut current)).await??;
        timeout(Duration::from_secs(1), stream.read_exact(&mut target)).await??;
        timeout(Duration::from_secs(1), stream.read_exact(&mut state)).await??;

        let current_temperature = LittleEndian::read_f32(&current);
        let target_temperature = LittleEndian::read_f32(&target);

        let current_temperature = if current_temperature.is_nan() {
            None
        } else {
            Some(current_temperature)
        };

        let target_temperature = if target_temperature.is_nan() {
            None
        } else {
            Some(target_temperature)
        };

        Ok(State {
            current_temperature,
            target_temperature,
            stirrer_on: (state[0] & RESPONSE_STIRRER_BIT) != 0,
            heater_on: (state[0] & RESPONSE_HEATER_BIT) != 0,
        })
    }

    /// Write a new target temperature in degree Celsius the Brewslave is supposed to reach.
    pub async fn set_temperature(&self, temperature: f32) -> Result<(), Error> {
        let mut command = vec![Command::SetTemperature as u8, 0, 0, 0, 0];
        LittleEndian::write_f32(&mut command[1..], temperature);

        let mut stream = self.stream.write().await;
        stream.write(&command).await?;
        ack_byte_to(stream.read_u8().await?)
    }

    /// Write new stirrer state.
    pub async fn write_stirrer(&self, stirrer_on: bool) -> Result<(), Error> {
        let command = match stirrer_on {
            true => Command::TurnStirrerOn as u8,
            false => Command::TurnStirrerOff as u8,
        };

        let mut stream = self.stream.write().await;
        stream.write_u8(command).await?;
        ack_byte_to(stream.read_u8().await?)
    }
}
