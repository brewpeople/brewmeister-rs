//! Serial communication with the Brewslave.

use anyhow::{anyhow, Result};
use byteorder::{ByteOrder, LittleEndian};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::RwLock;
use tokio_serial::{SerialPortBuilderExt, SerialStream};

/// Possible states of the stirrer.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StirrerState {
    On,
    Off,
}

/// Possible states of the heater.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HeaterState {
    On,
    Off,
}

/// Serial communication structure wrapping the Brewslave protocol.
pub struct Comm {
    stream: Arc<RwLock<SerialStream>>,
}

const RESPONSE_ACK: u8 = 0x80;
const RESPONSE_NACK: u8 = 0x40;

enum Command {
    ReadTemperature = 0x1,
    WriteTemperature = 0x2,
    ReadStirrer = 0x4,
    TurnStirrerOn = 0x5,
    TurnStirrerOff = 0x6,
    ReadHeater = 0x8,
}

fn ack_byte_to(ack: u8) -> Result<()> {
    if (ack | RESPONSE_ACK) != 0 {
        Ok(())
    } else if (ack | RESPONSE_NACK) != 0 {
        Err(anyhow!("Received NACK"))
    } else {
        Err(anyhow!("Unexpected response"))
    }
}

impl Comm {
    /// Create a new communication structure.
    ///
    /// As of now, it tries to open `/dev/tty/ACM0`.
    pub fn new() -> Result<Self> {
        let stream = tokio_serial::new("/dev/ttyACM0", 115200)
            .flow_control(tokio_serial::FlowControl::None)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::One)
            .open_native_async()?;

        Ok(Self {
            stream: Arc::new(RwLock::new(stream)),
        })
    }

    /// Read the current temperature in degree Celsius.
    pub async fn read_temperature(&self) -> Result<f32> {
        let mut stream = self.stream.write().await;
        stream.write_u8(Command::ReadTemperature as u8).await?;

        let mut float_bytes: [u8; 4] = [0; 4];
        stream.read_exact(&mut float_bytes).await?;
        // stream.read_buf(
        Ok(LittleEndian::read_f32(&float_bytes))
    }

    /// Write a new target temperature in degree Celsius the Brewslave is supposed to reach.
    pub async fn write_temperature(&self, temperature: f32) -> Result<()> {
        let mut command = vec![Command::WriteTemperature as u8, 0, 0, 0, 0];
        LittleEndian::write_f32(&mut command[1..], temperature);

        let mut stream = self.stream.write().await;
        stream.write(&command).await?;
        ack_byte_to(stream.read_u8().await?)
    }

    /// Read current stirrer state.
    pub async fn read_stirrer(&self) -> Result<StirrerState> {
        let mut stream = self.stream.write().await;
        stream.write_u8(Command::ReadStirrer as u8).await?;

        let response = stream.read_u8().await?;

        Ok(if response != 0 {
            StirrerState::On
        } else {
            StirrerState::Off
        })
    }

    /// Write new stirrer state.
    pub async fn write_stirrer(&self, state: StirrerState) -> Result<()> {
        let command = match state {
            StirrerState::On => Command::TurnStirrerOn as u8,
            StirrerState::Off => Command::TurnStirrerOff as u8,
        };

        let mut stream = self.stream.write().await;
        stream.write_u8(command).await?;
        ack_byte_to(stream.read_u8().await?)
    }

    /// Read heater state.
    pub async fn read_heater(&self) -> Result<HeaterState> {
        let mut stream = self.stream.write().await;
        stream.write_u8(Command::ReadHeater as u8).await?;

        let response = stream.read_u8().await?;

        Ok(if response != 0 {
            HeaterState::On
        } else {
            HeaterState::Off
        })
    }
}
