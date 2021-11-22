//! Serial communication with the Brewslave.

use anyhow::{anyhow, Result};
use byteorder::{ByteOrder, LittleEndian};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::RwLock;
use tokio_serial::{SerialPortBuilderExt, SerialStream};

/// Serial communication structure wrapping the Brewslave protocol.
pub struct Comm {
    stream: Arc<RwLock<SerialStream>>,
}

/// Current state of the Brewslave.
#[derive(Clone, Debug)]
pub struct State {
    pub temperature: f32,
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

    /// Read the current state comprised of temperature and device states.
    pub async fn read_state(&self) -> Result<State> {
        let mut stream = self.stream.write().await;
        stream.write_u8(Command::ReadState as u8).await?;

        let mut response: [u8; 5] = [0; 5];
        stream.read_exact(&mut response).await?;

        Ok(State {
            temperature: LittleEndian::read_f32(&response[0..4]),
            stirrer_on: (response[4] & RESPONSE_STIRRER_BIT) != 0,
            heater_on: (response[4] & RESPONSE_HEATER_BIT) != 0,
        })
    }

    /// Write a new target temperature in degree Celsius the Brewslave is supposed to reach.
    pub async fn set_temperature(&self, temperature: f32) -> Result<()> {
        let mut command = vec![Command::SetTemperature as u8, 0, 0, 0, 0];
        LittleEndian::write_f32(&mut command[1..], temperature);

        let mut stream = self.stream.write().await;
        stream.write(&command).await?;
        ack_byte_to(stream.read_u8().await?)
    }

    /// Write new stirrer state.
    pub async fn write_stirrer(&self, stirrer_on: bool) -> Result<()> {
        let command = match stirrer_on {
            true => Command::TurnStirrerOn as u8,
            false => Command::TurnStirrerOff as u8,
        };

        let mut stream = self.stream.write().await;
        stream.write_u8(command).await?;
        ack_byte_to(stream.read_u8().await?)
    }
}
