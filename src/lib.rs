#![warn(clippy::all, future_incompatible, nonstandard_style, rust_2018_idioms)]

use bleasy::common::characteristics::HEART_RATE_MEASUREMENT;
use bleasy::{Characteristic, Device, Error as BleError, ScanConfig, Scanner as BleScanner};
use futures::{Stream, StreamExt};
use std::pin::Pin;

#[derive(Debug)]
pub enum Error {
    /// The device did not have the wanted characteristic, or it could not be accessed
    CharacteristicNotFound,
    /// An error occurred in the underlying BLE library
    BleError(BleError),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Provides an interface for scanning heart rate sensors.
pub struct Scanner {
    ble_scanner: BleScanner,
}

impl Scanner {
    pub async fn new() -> Result<Self> {
        let ble_scanner = BleScanner::new().await.map_err(Error::BleError)?;

        Ok(Self { ble_scanner })
    }

    /// Starts scanning for heart rate sensors.
    pub async fn start(&mut self) -> Result<()> {
        let config = ScanConfig::default()
            .filter_by_characteristics(|chars| chars.contains(&HEART_RATE_MEASUREMENT))
            .stop_after_first_match();

        self.ble_scanner
            .start(config)
            .await
            .map_err(Error::BleError)
    }

    /// Waits for the next available heart rate sensor.
    /// None is returned if the scanner is stopped.
    pub async fn next_sensor(&mut self) -> Result<Option<Sensor>> {
        let device = match self.ble_scanner.device_stream().next().await {
            None => return Ok(None),
            Some(device) => device,
        };

        let characteristic = device
            .characteristic(HEART_RATE_MEASUREMENT)
            .await
            .map_err(Error::BleError)?
            .ok_or(Error::CharacteristicNotFound)?;

        Ok(Some(Sensor::new(device, characteristic)))
    }

    /// Stops scanning for BLE heart rate sensors.
    pub async fn stop(&mut self) -> Result<()> {
        self.ble_scanner.stop().await.map_err(Error::BleError)
    }
}

#[derive(Clone)]
pub struct Sensor {
    device: Device,
    characteristic: Characteristic,
}

impl Sensor {
    fn new(peripheral: Device, characteristic: Characteristic) -> Self {
        Self {
            device: peripheral,
            characteristic,
        }
    }

    /// Provides a stream of heart rate values as [`Option`]\<u8>.
    /// If the heart rate value is 0, [`Option::None`] is returned.
    pub async fn hr_stream(&mut self) -> Result<Pin<Box<dyn Stream<Item = Option<u8>> + Send>>> {
        let hr_stream = self
            .characteristic
            .subscribe()
            .await
            .map_err(Error::BleError)?;

        Ok(Box::pin(hr_stream.filter_map(move |n| async move {
            if n[1] == 0 {
                Some(None)
            } else {
                Some(Some(n[1]))
            }
        })))
    }

    pub async fn name(&self) -> Option<String> {
        self.device.local_name().await
    }
}
